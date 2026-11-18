use crate::auth::{AuthUser, MasterGuard};
use crate::db::log_audit;
use chrono::{DateTime, Utc};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post, put, State};
use serde_json::json;
use shared::models::{
    ApiError, DocumentPeriodDto, SignPeriodRequest, UpdatePeriodRequest, UserRole,
};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(FromRow)]
pub struct PeriodRow {
    pub id: Uuid,
    pub year: i32,
    pub title: String,
    pub subtitle: String,
    pub doc_number: String,
    pub revision: String,
    pub sk_basis: String,
    pub legal_basis: String,
    pub approval_date_text: String,
    pub chair_name: String,
    pub chair_nip: String,
    pub chair_signed_at: Option<DateTime<Utc>>,
    pub secretary_name: String,
    pub secretary_nip: String,
    pub secretary_verified_at: Option<DateTime<Utc>>,
    pub head_name: String,
    pub head_nip: String,
    pub head_approved_at: Option<DateTime<Utc>>,
    pub status: String,
}

impl PeriodRow {
    pub fn into_dto(self) -> DocumentPeriodDto {
        DocumentPeriodDto {
            id: self.id,
            year: self.year,
            title: self.title,
            subtitle: self.subtitle,
            doc_number: self.doc_number,
            revision: self.revision,
            sk_basis: self.sk_basis,
            legal_basis: self.legal_basis,
            approval_date_text: self.approval_date_text,
            chair_name: self.chair_name,
            chair_nip: self.chair_nip,
            chair_signed_at: self.chair_signed_at,
            secretary_name: self.secretary_name,
            secretary_nip: self.secretary_nip,
            secretary_verified_at: self.secretary_verified_at,
            head_name: self.head_name,
            head_nip: self.head_nip,
            head_approved_at: self.head_approved_at,
            status: self.status,
        }
    }
}

#[get("/api/periods")]
pub async fn list_periods(
    pool: &State<PgPool>,
    _auth: AuthUser,
) -> Result<Json<Vec<DocumentPeriodDto>>, (Status, Json<ApiError>)> {
    let rows = sqlx::query_as::<_, PeriodRow>(
        "SELECT id, year, title, subtitle, doc_number, revision, sk_basis, legal_basis,
                approval_date_text, chair_name, chair_nip, chair_signed_at,
                secretary_name, secretary_nip, secretary_verified_at,
                head_name, head_nip, head_approved_at, status
         FROM document_periods ORDER BY year DESC",
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;

    Ok(Json(rows.into_iter().map(|r| r.into_dto()).collect()))
}

#[put("/api/periods/<id>", data = "<req>")]
pub async fn update_period(
    pool: &State<PgPool>,
    master: MasterGuard,
    id: &str,
    req: Json<UpdatePeriodRequest>,
) -> Result<Json<DocumentPeriodDto>, (Status, Json<ApiError>)> {
    let period_id = Uuid::parse_str(id).map_err(|_| {
        (
            Status::BadRequest,
            Json(ApiError {
                message: "ID periode tidak valid".to_string(),
            }),
        )
    })?;

    let row = sqlx::query_as::<_, PeriodRow>(
        "UPDATE document_periods
         SET title = $1, subtitle = $2, doc_number = $3, revision = $4, sk_basis = $5,
             legal_basis = $6, approval_date_text = $7, chair_name = $8, chair_nip = $9,
             secretary_name = $10, secretary_nip = $11, head_name = $12, head_nip = $13,
             updated_at = NOW()
         WHERE id = $14
         RETURNING id, year, title, subtitle, doc_number, revision, sk_basis, legal_basis,
                   approval_date_text, chair_name, chair_nip, chair_signed_at,
                   secretary_name, secretary_nip, secretary_verified_at,
                   head_name, head_nip, head_approved_at, status",
    )
    .bind(&req.title)
    .bind(&req.subtitle)
    .bind(&req.doc_number)
    .bind(&req.revision)
    .bind(&req.sk_basis)
    .bind(&req.legal_basis)
    .bind(&req.approval_date_text)
    .bind(&req.chair_name)
    .bind(&req.chair_nip)
    .bind(&req.secretary_name)
    .bind(&req.secretary_nip)
    .bind(&req.head_name)
    .bind(&req.head_nip)
    .bind(period_id)
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;

    let Some(period) = row else {
        return Err((
            Status::NotFound,
            Json(ApiError {
                message: "Periode tidak ditemukan".to_string(),
            }),
        ));
    };

    log_audit(
        pool.inner(),
        Some(master.0.id),
        &master.0.username,
        "UPDATE_PERIOD",
        "document_period",
        Some(id),
        json!({ "doc_number": period.doc_number }),
    )
    .await;

    Ok(Json(period.into_dto()))
}

#[post("/api/periods/<id>/sign", data = "<req>")]
pub async fn sign_period(
    pool: &State<PgPool>,
    auth: AuthUser,
    id: &str,
    req: Json<SignPeriodRequest>,
) -> Result<Json<DocumentPeriodDto>, (Status, Json<ApiError>)> {
    let period_id = Uuid::parse_str(id).map_err(|_| {
        (
            Status::BadRequest,
            Json(ApiError {
                message: "ID periode tidak valid".to_string(),
            }),
        )
    })?;

    let sql = match req.stage.as_str() {
        "secretary" => {
            if !matches!(auth.role, UserRole::SekretarisTimK3 | UserRole::Admin) {
                return Err((
                    Status::Forbidden,
                    Json(ApiError {
                        message: "Hanya Sekretaris Tim K3 atau Admin yang dapat memverifikasi"
                            .to_string(),
                    }),
                ));
            }
            "UPDATE document_periods SET secretary_verified_at = NOW(), status = 'verified', updated_at = NOW() WHERE id = $1"
        }
        "chair" => {
            if !matches!(auth.role, UserRole::KetuaTimK3 | UserRole::Admin) {
                return Err((
                    Status::Forbidden,
                    Json(ApiError {
                        message: "Hanya Ketua Tim K3 atau Admin yang dapat menandatangani"
                            .to_string(),
                    }),
                ));
            }
            "UPDATE document_periods SET chair_signed_at = NOW(), status = 'signed', updated_at = NOW() WHERE id = $1"
        }
        "head" => {
            if !matches!(auth.role, UserRole::KetuaTimK3 | UserRole::Admin) {
                return Err((
                    Status::Forbidden,
                    Json(ApiError {
                        message: "Hanya Admin atau Ketua Tim K3 yang dapat mencatat persetujuan Kepala Balai"
                            .to_string(),
                    }),
                ));
            }
            "UPDATE document_periods SET head_approved_at = NOW(), status = 'approved', updated_at = NOW() WHERE id = $1"
        }
        _ => {
            return Err((
                Status::BadRequest,
                Json(ApiError {
                    message: "Tahap pengesahan tidak valid".to_string(),
                }),
            ))
        }
    };

    sqlx::query(sql)
        .bind(period_id)
        .execute(pool.inner())
        .await
        .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;

    let row = sqlx::query_as::<_, PeriodRow>(
        "SELECT id, year, title, subtitle, doc_number, revision, sk_basis, legal_basis,
                approval_date_text, chair_name, chair_nip, chair_signed_at,
                secretary_name, secretary_nip, secretary_verified_at,
                head_name, head_nip, head_approved_at, status
         FROM document_periods WHERE id = $1",
    )
    .bind(period_id)
    .fetch_one(pool.inner())
    .await
    .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;

    log_audit(
        pool.inner(),
        Some(auth.id),
        &auth.username,
        "SIGN_PERIOD",
        "document_period",
        Some(id),
        json!({ "stage": req.stage }),
    )
    .await;

    Ok(Json(row.into_dto()))
}
