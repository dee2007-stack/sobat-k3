use crate::auth::{create_jwt, verify_password, AuthUser};
use crate::db::log_audit;
use chrono::{DateTime, Utc};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post, State};
use serde_json::json;
use shared::models::{ApiError, LoginRequest, LoginResponse, UserDto, UserRole};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub full_name: String,
    pub nip: String,
    pub position_title: String,
    pub role: String,
    pub password_hash: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl UserRow {
    pub fn into_dto(self) -> UserDto {
        UserDto {
            id: self.id,
            username: self.username,
            email: self.email,
            full_name: self.full_name,
            nip: self.nip,
            position_title: self.position_title,
            role: UserRole::from_str(&self.role).unwrap_or(UserRole::AnggotaTimK3),
            is_active: self.is_active,
            created_at: self.created_at,
        }
    }
}

#[post("/api/auth/login", data = "<req>")]
pub async fn login(
    pool: &State<PgPool>,
    req: Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (Status, Json<ApiError>)> {
    let row = sqlx::query_as::<_, UserRow>(
        "SELECT id, username, email, full_name, nip, position_title, role, password_hash, is_active, created_at
         FROM users WHERE username = $1 OR email = $1",
    )
    .bind(req.username.trim())
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;

    let Some(user) = row else {
        return Err((
            Status::Unauthorized,
            Json(ApiError {
                message: "Username atau password salah".to_string(),
            }),
        ));
    };

    if !user.is_active {
        return Err((
            Status::Forbidden,
            Json(ApiError {
                message: "Akun dinonaktifkan oleh Admin".to_string(),
            }),
        ));
    }

    if !verify_password(&req.password, &user.password_hash) {
        return Err((
            Status::Unauthorized,
            Json(ApiError {
                message: "Username atau password salah".to_string(),
            }),
        ));
    }

    let role = UserRole::from_str(&user.role).unwrap_or(UserRole::AnggotaTimK3);
    let token = create_jwt(user.id, &user.username, role).map_err(|e| {
        (
            Status::InternalServerError,
            Json(ApiError { message: e }),
        )
    })?;

    log_audit(
        pool.inner(),
        Some(user.id),
        &user.username,
        "LOGIN",
        "user",
        Some(&user.id.to_string()),
        json!({ "role": role.as_str() }),
    )
    .await;

    Ok(Json(LoginResponse {
        token,
        user: user.into_dto(),
    }))
}

#[get("/api/auth/me")]
pub async fn me(
    pool: &State<PgPool>,
    auth: AuthUser,
) -> Result<Json<UserDto>, (Status, Json<ApiError>)> {
    let row = sqlx::query_as::<_, UserRow>(
        "SELECT id, username, email, full_name, nip, position_title, role, password_hash, is_active, created_at
         FROM users WHERE id = $1",
    )
    .bind(auth.id)
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;

    let Some(user) = row else {
        return Err((
            Status::NotFound,
            Json(ApiError {
                message: "Pengguna tidak ditemukan".to_string(),
            }),
        ));
    };

    Ok(Json(user.into_dto()))
}
