use crate::auth::{AuthUser, MasterGuard};
use crate::db::log_audit;
use crate::routes::work_hours::fetch_work_hours_summary;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, put, State};
use serde_json::json;
use shared::formulas::{calculate_fr, calculate_sr, calculate_trir};
use shared::models::{
    ApiError, LaggingIndicatorDto, LaggingMonthDto, UpdateLaggingMetaRequest,
    UpdateLaggingMonthRequest,
};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(FromRow)]
struct LaggingRow {
    id: Uuid,
    period_id: Uuid,
    item_number: i32,
    parameter_name: String,
    indicator_key: String,
    frequency: String,
    reference_formula: String,
    pic: String,
    notes: String,
    max_target: f64,
    is_auto_calculated: bool,
    root_cause_analysis: String,
    corrective_action: String,
}

#[derive(FromRow)]
struct LaggingMonthRow {
    indicator_id: Uuid,
    month: i32,
    value: f64,
    lost_days: i32,
    incident_notes: String,
}

pub async fn fetch_lagging_for_period(
    pool: &PgPool,
    period_id: Uuid,
) -> Result<Vec<LaggingIndicatorDto>, sqlx::Error> {
    let indicators = sqlx::query_as::<_, LaggingRow>(
        "SELECT id, period_id, item_number, parameter_name, indicator_key, frequency,
                reference_formula, pic, notes, max_target, is_auto_calculated,
                root_cause_analysis, corrective_action
         FROM lagging_indicators
         WHERE period_id = $1
         ORDER BY item_number ASC",
    )
    .bind(period_id)
    .fetch_all(pool)
    .await?;

    let month_rows = sqlx::query_as::<_, LaggingMonthRow>(
        "SELECT lm.indicator_id, lm.month, lm.value, lm.lost_days, lm.incident_notes
         FROM lagging_monthly lm
         JOIN lagging_indicators li ON li.id = lm.indicator_id
         WHERE li.period_id = $1
         ORDER BY lm.month ASC",
    )
    .bind(period_id)
    .fetch_all(pool)
    .await?;

    let work_hours = fetch_work_hours_summary(pool, period_id).await?;

    // Helper to get monthly values by indicator_key
    let get_monthly = |key: &str| -> [f64; 12] {
        let mut arr = [0.0; 12];
        if let Some(ind) = indicators.iter().find(|i| i.indicator_key == key) {
            for mr in month_rows.iter().filter(|m| m.indicator_id == ind.id) {
                if (1..=12).contains(&mr.month) {
                    arr[(mr.month - 1) as usize] = mr.value;
                }
            }
        }
        arr
    };

    let get_lost_days = || -> [i32; 12] {
        let mut arr = [0; 12];
        if let Some(ind) = indicators.iter().find(|i| i.indicator_key == "lti") {
            for mr in month_rows.iter().filter(|m| m.indicator_id == ind.id) {
                if (1..=12).contains(&mr.month) {
                    arr[(mr.month - 1) as usize] = mr.lost_days;
                }
            }
        }
        arr
    };

    let fatality_m = get_monthly("fatality");
    let lti_m = get_monthly("lti");
    let rwc_m = get_monthly("rwc");
    let mtc_m = get_monthly("mtc");
    let lost_days_m = get_lost_days();

    let mut manhours_m = [0.0; 12];
    for wh in &work_hours.months {
        if (1..=12).contains(&wh.month) {
            manhours_m[(wh.month - 1) as usize] = wh.cumulative_monthly_hours;
        }
    }
    let total_manhours: f64 = manhours_m.iter().sum();
    let total_fatality: f64 = fatality_m.iter().sum();
    let total_lti: f64 = lti_m.iter().sum();
    let total_rwc: f64 = rwc_m.iter().sum();
    let total_mtc: f64 = mtc_m.iter().sum();
    let total_lost_days: f64 = lost_days_m.iter().map(|&d| d as f64).sum();

    let mut result = Vec::with_capacity(indicators.len());
    for ind in indicators {
        let mut months: Vec<LaggingMonthDto> = Vec::with_capacity(12);
        let annual_realization: f64;

        if ind.indicator_key == "safe_manhours" {
            for m in 1..=12 {
                months.push(LaggingMonthDto {
                    month: m,
                    value: manhours_m[(m - 1) as usize],
                    lost_days: 0,
                    incident_notes: "Otomatis dari Tabel 1".to_string(),
                });
            }
            annual_realization = total_manhours;
        } else if ind.indicator_key == "fr" {
            for m in 1..=12 {
                let idx = (m - 1) as usize;
                months.push(LaggingMonthDto {
                    month: m,
                    value: calculate_fr(lti_m[idx], manhours_m[idx]),
                    lost_days: 0,
                    incident_notes: String::new(),
                });
            }
            annual_realization = calculate_fr(total_lti, total_manhours);
        } else if ind.indicator_key == "sr" {
            for m in 1..=12 {
                let idx = (m - 1) as usize;
                months.push(LaggingMonthDto {
                    month: m,
                    value: calculate_sr(lost_days_m[idx] as f64, manhours_m[idx]),
                    lost_days: lost_days_m[idx],
                    incident_notes: String::new(),
                });
            }
            annual_realization = calculate_sr(total_lost_days, total_manhours);
        } else if ind.indicator_key == "trir" {
            for m in 1..=12 {
                let idx = (m - 1) as usize;
                months.push(LaggingMonthDto {
                    month: m,
                    value: calculate_trir(
                        fatality_m[idx],
                        lti_m[idx],
                        rwc_m[idx],
                        mtc_m[idx],
                        manhours_m[idx],
                    ),
                    lost_days: 0,
                    incident_notes: String::new(),
                });
            }
            annual_realization =
                calculate_trir(total_fatality, total_lti, total_rwc, total_mtc, total_manhours);
        } else {
            for m in 1..=12 {
                let row = month_rows
                    .iter()
                    .find(|mr| mr.indicator_id == ind.id && mr.month == m);
                months.push(LaggingMonthDto {
                    month: m,
                    value: row.map(|r| r.value).unwrap_or(0.0),
                    lost_days: row.map(|r| r.lost_days).unwrap_or(0),
                    incident_notes: row.map(|r| r.incident_notes.clone()).unwrap_or_default(),
                });
            }
            annual_realization = months.iter().map(|m| m.value).sum();
        }

        let compliance_status = if ind.indicator_key == "safe_manhours" {
            "Terpantau".to_string()
        } else if ind.indicator_key == "nearmiss" {
            if annual_realization <= ind.max_target {
                "Tercapai (100%)".to_string()
            } else {
                "Perlu Evaluasi".to_string()
            }
        } else if annual_realization <= ind.max_target {
            "Tercapai (100%)".to_string()
        } else {
            "Tidak Tercapai".to_string()
        };

        result.push(LaggingIndicatorDto {
            id: ind.id,
            period_id: ind.period_id,
            item_number: ind.item_number,
            parameter_name: ind.parameter_name,
            indicator_key: ind.indicator_key,
            frequency: ind.frequency,
            reference_formula: ind.reference_formula,
            pic: ind.pic,
            notes: ind.notes,
            max_target: ind.max_target,
            annual_realization,
            compliance_status,
            is_auto_calculated: ind.is_auto_calculated,
            root_cause_analysis: ind.root_cause_analysis,
            corrective_action: ind.corrective_action,
            months,
        });
    }

    Ok(result)
}

#[get("/api/periods/<id>/lagging")]
pub async fn list_lagging(
    pool: &State<PgPool>,
    _auth: AuthUser,
    id: &str,
) -> Result<Json<Vec<LaggingIndicatorDto>>, (Status, Json<ApiError>)> {
    let period_id = Uuid::parse_str(id).map_err(|_| {
        (
            Status::BadRequest,
            Json(ApiError {
                message: "ID periode tidak valid".to_string(),
            }),
        )
    })?;

    let list = fetch_lagging_for_period(pool.inner(), period_id)
        .await
        .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;

    Ok(Json(list))
}

#[put("/api/lagging/<id>", data = "<req>")]
pub async fn update_lagging_meta(
    pool: &State<PgPool>,
    master: MasterGuard,
    id: &str,
    req: Json<UpdateLaggingMetaRequest>,
) -> Result<Json<LaggingIndicatorDto>, (Status, Json<ApiError>)> {
    let ind_id = Uuid::parse_str(id).map_err(|_| {
        (
            Status::BadRequest,
            Json(ApiError {
                message: "ID indikator tidak valid".to_string(),
            }),
        )
    })?;

    let row: Option<(Uuid,)> = sqlx::query_as(
        "UPDATE lagging_indicators
         SET root_cause_analysis = $1, corrective_action = $2
         WHERE id = $3
         RETURNING period_id",
    )
    .bind(&req.root_cause_analysis)
    .bind(&req.corrective_action)
    .bind(ind_id)
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;

    let Some((period_id,)) = row else {
        return Err((
            Status::NotFound,
            Json(ApiError {
                message: "Indikator tidak ditemukan".to_string(),
            }),
        ));
    };

    log_audit(
        pool.inner(),
        Some(master.0.id),
        &master.0.username,
        "UPDATE_LAGGING_META",
        "lagging_indicator",
        Some(id),
        json!({}),
    )
    .await;

    let all = fetch_lagging_for_period(pool.inner(), period_id)
        .await
        .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;
    let updated = all.into_iter().find(|i| i.id == ind_id).ok_or((
        Status::NotFound,
        Json(ApiError {
            message: "Indikator tidak ditemukan".to_string(),
        }),
    ))?;
    Ok(Json(updated))
}

#[put("/api/lagging/<id>/months/<month>", data = "<req>")]
pub async fn update_lagging_month(
    pool: &State<PgPool>,
    auth: AuthUser,
    id: &str,
    month: i32,
    req: Json<UpdateLaggingMonthRequest>,
) -> Result<Json<LaggingIndicatorDto>, (Status, Json<ApiError>)> {
    if !(1..=12).contains(&month) {
        return Err((
            Status::BadRequest,
            Json(ApiError {
                message: "Bulan harus 1..12".to_string(),
            }),
        ));
    }

    let ind_id = Uuid::parse_str(id).map_err(|_| {
        (
            Status::BadRequest,
            Json(ApiError {
                message: "ID indikator tidak valid".to_string(),
            }),
        )
    })?;

    let ind_info: Option<(Uuid, bool)> = sqlx::query_as(
        "SELECT period_id, is_auto_calculated FROM lagging_indicators WHERE id = $1",
    )
    .bind(ind_id)
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;

    let Some((period_id, is_auto)) = ind_info else {
        return Err((
            Status::NotFound,
            Json(ApiError {
                message: "Indikator tidak ditemukan".to_string(),
            }),
        ));
    };

    if is_auto {
        return Err((
            Status::BadRequest,
            Json(ApiError {
                message: "Indikator ini dihitung otomatis dari Tabel 1 & Insiden".to_string(),
            }),
        ));
    }

    sqlx::query(
        "UPDATE lagging_monthly
         SET value = $1, lost_days = $2, incident_notes = $3, updated_by = $4, updated_at = NOW()
         WHERE indicator_id = $5 AND month = $6",
    )
    .bind(req.value.max(0.0))
    .bind(req.lost_days.max(0))
    .bind(&req.incident_notes)
    .bind(auth.id)
    .bind(ind_id)
    .bind(month)
    .execute(pool.inner())
    .await
    .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;

    log_audit(
        pool.inner(),
        Some(auth.id),
        &auth.username,
        "UPDATE_LAGGING_MONTH",
        "lagging_monthly",
        Some(&format!("{}:{}", id, month)),
        json!({ "value": req.value, "lost_days": req.lost_days }),
    )
    .await;

    let all = fetch_lagging_for_period(pool.inner(), period_id)
        .await
        .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;
    let updated = all.into_iter().find(|i| i.id == ind_id).ok_or((
        Status::NotFound,
        Json(ApiError {
            message: "Indikator tidak ditemukan".to_string(),
        }),
    ))?;
    Ok(Json(updated))
}
