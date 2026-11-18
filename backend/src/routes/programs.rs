use crate::auth::{AuthUser, MasterGuard};
use crate::db::log_audit;
use chrono::{DateTime, Utc};
use rocket::data::{Data, ToByteUnit};
use rocket::fs::NamedFile;
use rocket::http::{ContentType, Status};
use rocket::serde::json::Json;
use rocket::{delete, get, post, put, State};
use serde_json::json;
use shared::formulas::calculate_achievement_pct;
use shared::models::{
    ApiError, EvidenceFileDto, UpdateProgramMonthRequest, UpsertWorkProgramRequest,
    WorkProgramDto, WorkProgramMonthDto,
};
use sqlx::{FromRow, PgPool};
use std::env;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(FromRow)]
struct ProgramRow {
    id: Uuid,
    period_id: Uuid,
    pillar: String,
    pillar_title: String,
    subcategory: String,
    item_number: String,
    objective: String,
    measurable_target: String,
    code: String,
    description: String,
    frequency: String,
    reference_law: String,
    pic: String,
    output_evidence: String,
    annual_target: f64,
    is_target_auto_sum: bool,
    root_cause_analysis: String,
    corrective_preventive_action: String,
    sort_order: i32,
}

#[derive(FromRow)]
struct ProgramMonthRow {
    program_id: Uuid,
    month: i32,
    plan_label: String,
    plan_value: f64,
    actual_value: f64,
    notes: String,
}

#[derive(FromRow)]
struct EvidenceRow {
    id: Uuid,
    program_id: Uuid,
    month: Option<i32>,
    file_name: String,
    mime_type: String,
    file_size: i64,
    uploaded_at: DateTime<Utc>,
}

pub async fn fetch_programs_for_period(
    pool: &PgPool,
    period_id: Uuid,
) -> Result<Vec<WorkProgramDto>, sqlx::Error> {
    let programs = sqlx::query_as::<_, ProgramRow>(
        "SELECT id, period_id, pillar, pillar_title, subcategory, item_number, objective,
                measurable_target, code, description, frequency, reference_law, pic,
                output_evidence, annual_target, is_target_auto_sum, root_cause_analysis,
                corrective_preventive_action, sort_order
         FROM work_programs
         WHERE period_id = $1
         ORDER BY pillar ASC, sort_order ASC, code ASC",
    )
    .bind(period_id)
    .fetch_all(pool)
    .await?;

    let month_rows = sqlx::query_as::<_, ProgramMonthRow>(
        "SELECT wpm.program_id, wpm.month, wpm.plan_label, wpm.plan_value, wpm.actual_value, wpm.notes
         FROM work_program_monthly wpm
         JOIN work_programs wp ON wp.id = wpm.program_id
         WHERE wp.period_id = $1
         ORDER BY wpm.month ASC",
    )
    .bind(period_id)
    .fetch_all(pool)
    .await?;

    let evidence_rows = sqlx::query_as::<_, EvidenceRow>(
        "SELECT ef.id, ef.program_id, ef.month, ef.file_name, ef.mime_type, ef.file_size, ef.uploaded_at
         FROM evidence_files ef
         JOIN work_programs wp ON wp.id = ef.program_id
         WHERE wp.period_id = $1
         ORDER BY ef.uploaded_at DESC",
    )
    .bind(period_id)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::with_capacity(programs.len());
    for p in programs {
        let months: Vec<WorkProgramMonthDto> = month_rows
            .iter()
            .filter(|m| m.program_id == p.id)
            .map(|m| WorkProgramMonthDto {
                month: m.month,
                plan_label: m.plan_label.clone(),
                plan_value: m.plan_value,
                actual_value: m.actual_value,
                notes: m.notes.clone(),
            })
            .collect();

        let evidences: Vec<EvidenceFileDto> = evidence_rows
            .iter()
            .filter(|e| e.program_id == p.id)
            .map(|e| EvidenceFileDto {
                id: e.id,
                program_id: e.program_id,
                month: e.month,
                file_name: e.file_name.clone(),
                mime_type: e.mime_type.clone(),
                file_size: e.file_size,
                uploaded_at: e.uploaded_at,
            })
            .collect();

        let target = if p.is_target_auto_sum {
            months.iter().map(|m| m.plan_value).sum()
        } else {
            p.annual_target
        };
        let realization: f64 = months.iter().map(|m| m.actual_value).sum();
        let pct = calculate_achievement_pct(realization, target);

        result.push(WorkProgramDto {
            id: p.id,
            period_id: p.period_id,
            pillar: p.pillar,
            pillar_title: p.pillar_title,
            subcategory: p.subcategory,
            item_number: p.item_number,
            objective: p.objective,
            measurable_target: p.measurable_target,
            code: p.code,
            description: p.description,
            frequency: p.frequency,
            reference_law: p.reference_law,
            pic: p.pic,
            output_evidence: p.output_evidence,
            annual_target: target,
            annual_realization: realization,
            achievement_pct: pct,
            is_target_auto_sum: p.is_target_auto_sum,
            root_cause_analysis: p.root_cause_analysis,
            corrective_preventive_action: p.corrective_preventive_action,
            sort_order: p.sort_order,
            months,
            evidences,
        });
    }

    Ok(result)
}

#[get("/api/periods/<id>/programs")]
pub async fn list_programs(
    pool: &State<PgPool>,
    _auth: AuthUser,
    id: &str,
) -> Result<Json<Vec<WorkProgramDto>>, (Status, Json<ApiError>)> {
    let period_id = Uuid::parse_str(id).map_err(|_| {
        (
            Status::BadRequest,
            Json(ApiError {
                message: "ID periode tidak valid".to_string(),
            }),
        )
    })?;

    let list = fetch_programs_for_period(pool.inner(), period_id)
        .await
        .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;

    Ok(Json(list))
}

#[post("/api/periods/<id>/programs", data = "<req>")]
pub async fn create_program(
    pool: &State<PgPool>,
    master: MasterGuard,
    id: &str,
    req: Json<UpsertWorkProgramRequest>,
) -> Result<Json<WorkProgramDto>, (Status, Json<ApiError>)> {
    let period_id = Uuid::parse_str(id).map_err(|_| {
        (
            Status::BadRequest,
            Json(ApiError {
                message: "ID periode tidak valid".to_string(),
            }),
        )
    })?;

    let prog_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO work_programs (
            id, period_id, pillar, pillar_title, subcategory, item_number, objective,
            measurable_target, code, description, frequency, reference_law, pic,
            output_evidence, annual_target, is_target_auto_sum, root_cause_analysis,
            corrective_preventive_action, sort_order
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19)",
    )
    .bind(prog_id)
    .bind(period_id)
    .bind(&req.pillar)
    .bind(&req.pillar_title)
    .bind(&req.subcategory)
    .bind(&req.item_number)
    .bind(&req.objective)
    .bind(&req.measurable_target)
    .bind(&req.code)
    .bind(&req.description)
    .bind(&req.frequency)
    .bind(&req.reference_law)
    .bind(&req.pic)
    .bind(&req.output_evidence)
    .bind(req.annual_target)
    .bind(req.is_target_auto_sum)
    .bind(&req.root_cause_analysis)
    .bind(&req.corrective_preventive_action)
    .bind(req.sort_order)
    .execute(pool.inner())
    .await
    .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;

    for m in 1..=12 {
        let month_data = req.months.iter().find(|x| x.month == m);
        let (label, pval, aval, notes) = match month_data {
            Some(md) => (
                md.plan_label.as_str(),
                md.plan_value,
                md.actual_value,
                md.notes.as_str(),
            ),
            None => ("", 0.0, 0.0, ""),
        };
        sqlx::query(
            "INSERT INTO work_program_monthly (program_id, month, plan_label, plan_value, actual_value, notes, updated_by)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(prog_id)
        .bind(m)
        .bind(label)
        .bind(pval)
        .bind(aval)
        .bind(notes)
        .bind(master.0.id)
        .execute(pool.inner())
        .await
        .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;
    }

    log_audit(
        pool.inner(),
        Some(master.0.id),
        &master.0.username,
        "CREATE_PROGRAM",
        "work_program",
        Some(&prog_id.to_string()),
        json!({ "code": req.code }),
    )
    .await;

    let all = fetch_programs_for_period(pool.inner(), period_id)
        .await
        .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;
    let created = all
        .into_iter()
        .find(|p| p.id == prog_id)
        .ok_or((
            Status::InternalServerError,
            Json(ApiError {
                message: "Program tidak ditemukan setelah dibuat".to_string(),
            }),
        ))?;
    Ok(Json(created))
}

#[put("/api/programs/<id>", data = "<req>")]
pub async fn update_program(
    pool: &State<PgPool>,
    master: MasterGuard,
    id: &str,
    req: Json<UpsertWorkProgramRequest>,
) -> Result<Json<WorkProgramDto>, (Status, Json<ApiError>)> {
    let prog_id = Uuid::parse_str(id).map_err(|_| {
        (
            Status::BadRequest,
            Json(ApiError {
                message: "ID program tidak valid".to_string(),
            }),
        )
    })?;

    let period_row: Option<(Uuid,)> = sqlx::query_as(
        "UPDATE work_programs
         SET pillar = $1, pillar_title = $2, subcategory = $3, item_number = $4,
             objective = $5, measurable_target = $6, code = $7, description = $8,
             frequency = $9, reference_law = $10, pic = $11, output_evidence = $12,
             annual_target = $13, is_target_auto_sum = $14, root_cause_analysis = $15,
             corrective_preventive_action = $16, sort_order = $17, updated_at = NOW()
         WHERE id = $18
         RETURNING period_id",
    )
    .bind(&req.pillar)
    .bind(&req.pillar_title)
    .bind(&req.subcategory)
    .bind(&req.item_number)
    .bind(&req.objective)
    .bind(&req.measurable_target)
    .bind(&req.code)
    .bind(&req.description)
    .bind(&req.frequency)
    .bind(&req.reference_law)
    .bind(&req.pic)
    .bind(&req.output_evidence)
    .bind(req.annual_target)
    .bind(req.is_target_auto_sum)
    .bind(&req.root_cause_analysis)
    .bind(&req.corrective_preventive_action)
    .bind(req.sort_order)
    .bind(prog_id)
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;

    let Some((period_id,)) = period_row else {
        return Err((
            Status::NotFound,
            Json(ApiError {
                message: "Program tidak ditemukan".to_string(),
            }),
        ));
    };

    for md in &req.months {
        sqlx::query(
            "UPDATE work_program_monthly
             SET plan_label = $1, plan_value = $2, actual_value = $3, notes = $4, updated_by = $5, updated_at = NOW()
             WHERE program_id = $6 AND month = $7",
        )
        .bind(&md.plan_label)
        .bind(md.plan_value)
        .bind(md.actual_value)
        .bind(&md.notes)
        .bind(master.0.id)
        .bind(prog_id)
        .bind(md.month)
        .execute(pool.inner())
        .await
        .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;
    }

    log_audit(
        pool.inner(),
        Some(master.0.id),
        &master.0.username,
        "UPDATE_PROGRAM",
        "work_program",
        Some(id),
        json!({ "code": req.code }),
    )
    .await;

    let all = fetch_programs_for_period(pool.inner(), period_id)
        .await
        .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;
    let updated = all.into_iter().find(|p| p.id == prog_id).ok_or((
        Status::NotFound,
        Json(ApiError {
            message: "Program tidak ditemukan".to_string(),
        }),
    ))?;
    Ok(Json(updated))
}

#[put("/api/programs/<id>/months/<month>", data = "<req>")]
pub async fn update_program_month(
    pool: &State<PgPool>,
    auth: AuthUser,
    id: &str,
    month: i32,
    req: Json<UpdateProgramMonthRequest>,
) -> Result<Json<WorkProgramDto>, (Status, Json<ApiError>)> {
    if !(1..=12).contains(&month) {
        return Err((
            Status::BadRequest,
            Json(ApiError {
                message: "Bulan harus 1..12".to_string(),
            }),
        ));
    }

    let prog_id = Uuid::parse_str(id).map_err(|_| {
        (
            Status::BadRequest,
            Json(ApiError {
                message: "ID program tidak valid".to_string(),
            }),
        )
    })?;

    if auth.role.can_manage_master() {
        if let (Some(label), Some(pval)) = (&req.plan_label, req.plan_value) {
            sqlx::query(
                "UPDATE work_program_monthly
                 SET plan_label = $1, plan_value = $2, actual_value = $3, notes = $4, updated_by = $5, updated_at = NOW()
                 WHERE program_id = $6 AND month = $7",
            )
            .bind(label)
            .bind(pval)
            .bind(req.actual_value.max(0.0))
            .bind(&req.notes)
            .bind(auth.id)
            .bind(prog_id)
            .bind(month)
            .execute(pool.inner())
            .await
            .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;
        } else {
            sqlx::query(
                "UPDATE work_program_monthly
                 SET actual_value = $1, notes = $2, updated_by = $3, updated_at = NOW()
                 WHERE program_id = $4 AND month = $5",
            )
            .bind(req.actual_value.max(0.0))
            .bind(&req.notes)
            .bind(auth.id)
            .bind(prog_id)
            .bind(month)
            .execute(pool.inner())
            .await
            .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;
        }
    } else {
        sqlx::query(
            "UPDATE work_program_monthly
             SET actual_value = $1, notes = $2, updated_by = $3, updated_at = NOW()
             WHERE program_id = $4 AND month = $5",
        )
        .bind(req.actual_value.max(0.0))
        .bind(&req.notes)
        .bind(auth.id)
        .bind(prog_id)
        .bind(month)
        .execute(pool.inner())
        .await
        .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;
    }

    let (period_id,): (Uuid,) =
        sqlx::query_as("SELECT period_id FROM work_programs WHERE id = $1")
            .bind(prog_id)
            .fetch_one(pool.inner())
            .await
            .map_err(|e| (Status::NotFound, Json(ApiError { message: e.to_string() })))?;

    log_audit(
        pool.inner(),
        Some(auth.id),
        &auth.username,
        "UPDATE_PROGRAM_MONTH",
        "work_program_monthly",
        Some(&format!("{}:{}", id, month)),
        json!({ "actual_value": req.actual_value }),
    )
    .await;

    let all = fetch_programs_for_period(pool.inner(), period_id)
        .await
        .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;
    let updated = all.into_iter().find(|p| p.id == prog_id).ok_or((
        Status::NotFound,
        Json(ApiError {
            message: "Program tidak ditemukan".to_string(),
        }),
    ))?;
    Ok(Json(updated))
}

#[delete("/api/programs/<id>")]
pub async fn delete_program(
    pool: &State<PgPool>,
    master: MasterGuard,
    id: &str,
) -> Result<Json<serde_json::Value>, (Status, Json<ApiError>)> {
    let prog_id = Uuid::parse_str(id).map_err(|_| {
        (
            Status::BadRequest,
            Json(ApiError {
                message: "ID program tidak valid".to_string(),
            }),
        )
    })?;

    sqlx::query("DELETE FROM work_programs WHERE id = $1")
        .bind(prog_id)
        .execute(pool.inner())
        .await
        .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;

    log_audit(
        pool.inner(),
        Some(master.0.id),
        &master.0.username,
        "DELETE_PROGRAM",
        "work_program",
        Some(id),
        json!({}),
    )
    .await;

    Ok(Json(json!({ "ok": true })))
}

#[post("/api/programs/<id>/evidence?<filename>&<month>", data = "<data>")]
pub async fn upload_evidence(
    pool: &State<PgPool>,
    auth: AuthUser,
    id: &str,
    filename: &str,
    month: Option<i32>,
    content_type: Option<&ContentType>,
    data: Data<'_>,
) -> Result<Json<EvidenceFileDto>, (Status, Json<ApiError>)> {
    let prog_id = Uuid::parse_str(id).map_err(|_| {
        (
            Status::BadRequest,
            Json(ApiError {
                message: "ID program tidak valid".to_string(),
            }),
        )
    })?;

    let upload_dir = env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string());
    let dir_path = PathBuf::from(&upload_dir);
    tokio::fs::create_dir_all(&dir_path)
        .await
        .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;

    let file_id = Uuid::new_v4();
    let safe_name: String = filename
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let disk_name = format!("{}_{}", file_id, safe_name);
    let full_path = dir_path.join(&disk_name);

    let bytes = data
        .open(10.mebibytes())
        .into_bytes()
        .await
        .map_err(|e| (Status::BadRequest, Json(ApiError { message: e.to_string() })))?;

    if !bytes.is_complete() {
        return Err((
            Status::PayloadTooLarge,
            Json(ApiError {
                message: "Ukuran file maksimal 10 MB".to_string(),
            }),
        ));
    }

    let raw = bytes.into_inner();
    let file_size = raw.len() as i64;
    tokio::fs::write(&full_path, &raw)
        .await
        .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;

    let mime = content_type
        .map(|c| c.to_string())
        .unwrap_or_else(|| "application/octet-stream".to_string());

    let row = sqlx::query_as::<_, EvidenceRow>(
        "INSERT INTO evidence_files (id, program_id, month, file_name, file_path, mime_type, file_size, uploaded_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING id, program_id, month, file_name, mime_type, file_size, uploaded_at",
    )
    .bind(file_id)
    .bind(prog_id)
    .bind(month)
    .bind(&safe_name)
    .bind(full_path.to_string_lossy().to_string())
    .bind(&mime)
    .bind(file_size)
    .bind(auth.id)
    .fetch_one(pool.inner())
    .await
    .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;

    Ok(Json(EvidenceFileDto {
        id: row.id,
        program_id: row.program_id,
        month: row.month,
        file_name: row.file_name,
        mime_type: row.mime_type,
        file_size: row.file_size,
        uploaded_at: row.uploaded_at,
    }))
}

#[get("/api/evidence/<file_id>")]
pub async fn download_evidence(
    pool: &State<PgPool>,
    file_id: &str,
) -> Result<NamedFile, Status> {
    let fid = Uuid::parse_str(file_id).map_err(|_| Status::BadRequest)?;
    let row: Option<(String,)> =
        sqlx::query_as("SELECT file_path FROM evidence_files WHERE id = $1")
            .bind(fid)
            .fetch_optional(pool.inner())
            .await
            .map_err(|_| Status::InternalServerError)?;

    let Some((path,)) = row else {
        return Err(Status::NotFound);
    };
    NamedFile::open(path).await.map_err(|_| Status::NotFound)
}
