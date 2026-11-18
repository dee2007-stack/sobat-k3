use crate::auth::{hash_password, AdminGuard};
use crate::db::log_audit;
use crate::routes::auth::UserRow;
use chrono::{DateTime, Utc};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{delete, get, post, put, State};
use serde_json::json;
use shared::models::{
    ApiError, AuditLogDto, CreateUserRequest, ResetPasswordRequest, UpdateUserRequest, UserDto,
};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[get("/api/users")]
pub async fn list_users(
    pool: &State<PgPool>,
    _admin: AdminGuard,
) -> Result<Json<Vec<UserDto>>, (Status, Json<ApiError>)> {
    let rows = sqlx::query_as::<_, UserRow>(
        "SELECT id, username, email, full_name, nip, position_title, role, password_hash, is_active, created_at
         FROM users ORDER BY created_at ASC",
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;

    Ok(Json(rows.into_iter().map(|r| r.into_dto()).collect()))
}

#[post("/api/users", data = "<req>")]
pub async fn create_user(
    pool: &State<PgPool>,
    admin: AdminGuard,
    req: Json<CreateUserRequest>,
) -> Result<Json<UserDto>, (Status, Json<ApiError>)> {
    if req.username.trim().is_empty() || req.password.len() < 6 {
        return Err((
            Status::BadRequest,
            Json(ApiError {
                message: "Username wajib diisi dan password minimal 6 karakter".to_string(),
            }),
        ));
    }

    let pw_hash = hash_password(&req.password).map_err(|e| {
        (
            Status::InternalServerError,
            Json(ApiError { message: e }),
        )
    })?;

    let row = sqlx::query_as::<_, UserRow>(
        "INSERT INTO users (username, email, full_name, nip, position_title, role, password_hash)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING id, username, email, full_name, nip, position_title, role, password_hash, is_active, created_at",
    )
    .bind(req.username.trim())
    .bind(req.email.trim())
    .bind(req.full_name.trim())
    .bind(req.nip.trim())
    .bind(req.position_title.trim())
    .bind(req.role.as_str())
    .bind(pw_hash)
    .fetch_one(pool.inner())
    .await
    .map_err(|e| {
        (
            Status::BadRequest,
            Json(ApiError {
                message: format!("Gagal membuat pengguna (username/email mungkin sudah ada): {}", e),
            }),
        )
    })?;

    let dto = row.into_dto();
    log_audit(
        pool.inner(),
        Some(admin.0.id),
        &admin.0.username,
        "CREATE_USER",
        "user",
        Some(&dto.id.to_string()),
        json!({ "username": dto.username, "role": dto.role.as_str() }),
    )
    .await;

    Ok(Json(dto))
}

#[put("/api/users/<id>", data = "<req>")]
pub async fn update_user(
    pool: &State<PgPool>,
    admin: AdminGuard,
    id: &str,
    req: Json<UpdateUserRequest>,
) -> Result<Json<UserDto>, (Status, Json<ApiError>)> {
    let user_id = Uuid::parse_str(id).map_err(|_| {
        (
            Status::BadRequest,
            Json(ApiError {
                message: "ID pengguna tidak valid".to_string(),
            }),
        )
    })?;

    let row = sqlx::query_as::<_, UserRow>(
        "UPDATE users
         SET email = $1, full_name = $2, nip = $3, position_title = $4, role = $5, is_active = $6, updated_at = NOW()
         WHERE id = $7
         RETURNING id, username, email, full_name, nip, position_title, role, password_hash, is_active, created_at",
    )
    .bind(req.email.trim())
    .bind(req.full_name.trim())
    .bind(req.nip.trim())
    .bind(req.position_title.trim())
    .bind(req.role.as_str())
    .bind(req.is_active)
    .bind(user_id)
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| {
        (
            Status::BadRequest,
            Json(ApiError {
                message: format!("Gagal memperbarui pengguna (email mungkin sudah digunakan): {}", e),
            }),
        )
    })?;

    let Some(user) = row else {
        return Err((
            Status::NotFound,
            Json(ApiError {
                message: "Pengguna tidak ditemukan".to_string(),
            }),
        ));
    };

    let dto = user.into_dto();
    log_audit(
        pool.inner(),
        Some(admin.0.id),
        &admin.0.username,
        "UPDATE_USER",
        "user",
        Some(&dto.id.to_string()),
        json!({ "role": dto.role.as_str(), "is_active": dto.is_active }),
    )
    .await;

    Ok(Json(dto))
}

#[post("/api/users/<id>/reset-password", data = "<req>")]
pub async fn reset_password(
    pool: &State<PgPool>,
    admin: AdminGuard,
    id: &str,
    req: Json<ResetPasswordRequest>,
) -> Result<Json<serde_json::Value>, (Status, Json<ApiError>)> {
    if req.new_password.len() < 6 {
        return Err((
            Status::BadRequest,
            Json(ApiError {
                message: "Password baru minimal 6 karakter".to_string(),
            }),
        ));
    }
    let user_id = Uuid::parse_str(id).map_err(|_| {
        (
            Status::BadRequest,
            Json(ApiError {
                message: "ID pengguna tidak valid".to_string(),
            }),
        )
    })?;
    let pw_hash = hash_password(&req.new_password).map_err(|e| {
        (
            Status::InternalServerError,
            Json(ApiError { message: e }),
        )
    })?;

    let res = sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
        .bind(pw_hash)
        .bind(user_id)
        .execute(pool.inner())
        .await
        .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;

    if res.rows_affected() == 0 {
        return Err((
            Status::NotFound,
            Json(ApiError {
                message: "Pengguna tidak ditemukan".to_string(),
            }),
        ));
    }

    log_audit(
        pool.inner(),
        Some(admin.0.id),
        &admin.0.username,
        "RESET_PASSWORD",
        "user",
        Some(id),
        json!({}),
    )
    .await;

    Ok(Json(json!({ "ok": true })))
}

#[delete("/api/users/<id>")]
pub async fn delete_user(
    pool: &State<PgPool>,
    admin: AdminGuard,
    id: &str,
) -> Result<Json<serde_json::Value>, (Status, Json<ApiError>)> {
    let user_id = Uuid::parse_str(id).map_err(|_| {
        (
            Status::BadRequest,
            Json(ApiError {
                message: "ID pengguna tidak valid".to_string(),
            }),
        )
    })?;

    if user_id == admin.0.id {
        return Err((
            Status::BadRequest,
            Json(ApiError {
                message: "Tidak dapat menghapus akun sendiri".to_string(),
            }),
        ));
    }

    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(pool.inner())
        .await
        .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;

    log_audit(
        pool.inner(),
        Some(admin.0.id),
        &admin.0.username,
        "DELETE_USER",
        "user",
        Some(id),
        json!({}),
    )
    .await;

    Ok(Json(json!({ "ok": true })))
}

#[derive(FromRow)]
struct AuditRow {
    id: Uuid,
    user_id: Option<Uuid>,
    username: String,
    action: String,
    entity_type: String,
    entity_id: Option<String>,
    details: serde_json::Value,
    created_at: DateTime<Utc>,
}

#[get("/api/audit-logs")]
pub async fn list_audit_logs(
    pool: &State<PgPool>,
    _admin: AdminGuard,
) -> Result<Json<Vec<AuditLogDto>>, (Status, Json<ApiError>)> {
    let rows = sqlx::query_as::<_, AuditRow>(
        "SELECT id, user_id, username, action, entity_type, entity_id, details, created_at
         FROM audit_logs ORDER BY created_at DESC LIMIT 100",
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;

    Ok(Json(
        rows.into_iter()
            .map(|r| AuditLogDto {
                id: r.id,
                user_id: r.user_id,
                username: r.username,
                action: r.action,
                entity_type: r.entity_type,
                entity_id: r.entity_id,
                details: r.details,
                created_at: r.created_at,
            })
            .collect(),
    ))
}
