use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use serde::{Deserialize, Serialize};
use shared::models::UserRole;
use std::env;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub role: String,
    pub exp: usize,
}

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| format!("Hash error: {}", e))
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    let Ok(parsed_hash) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

fn jwt_secret() -> String {
    env::var("JWT_SECRET").unwrap_or_else(|_| {
        "sobat_k3_bbpom_bandung_super_secret_key_minimal_32_karakter_2027".to_string()
    })
}

pub fn create_jwt(user_id: Uuid, username: &str, role: UserRole) -> Result<String, String> {
    let expiration = (Utc::now() + Duration::hours(24)).timestamp() as usize;
    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        role: role.as_str().to_string(),
        exp: expiration,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
    .map_err(|e| format!("JWT encode error: {}", e))
}

pub fn decode_jwt(token: &str) -> Result<Claims, String> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| format!("JWT decode error: {}", e))
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub username: String,
    pub role: UserRole,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = &'static str;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth_header = req.headers().get_one("Authorization");
        let token = match auth_header {
            Some(h) if h.starts_with("Bearer ") => &h[7..],
            _ => return Outcome::Error((Status::Unauthorized, "Token tidak ditemukan")),
        };

        let Ok(claims) = decode_jwt(token) else {
            return Outcome::Error((Status::Unauthorized, "Token tidak valid atau kedaluwarsa"));
        };

        let Ok(id) = Uuid::parse_str(&claims.sub) else {
            return Outcome::Error((Status::Unauthorized, "User ID tidak valid"));
        };

        let Some(role) = UserRole::from_str(&claims.role) else {
            return Outcome::Error((Status::Unauthorized, "Role tidak valid"));
        };

        Outcome::Success(AuthUser {
            id,
            username: claims.username,
            role,
        })
    }
}

pub struct AdminGuard(pub AuthUser);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminGuard {
    type Error = &'static str;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match AuthUser::from_request(req).await {
            Outcome::Success(user) if user.role.can_manage_users() => {
                Outcome::Success(AdminGuard(user))
            }
            Outcome::Success(_) => Outcome::Error((Status::Forbidden, "Khusus Admin")),
            Outcome::Error(e) => Outcome::Error(e),
            Outcome::Forward(f) => Outcome::Forward(f),
        }
    }
}

pub struct MasterGuard(pub AuthUser);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for MasterGuard {
    type Error = &'static str;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match AuthUser::from_request(req).await {
            Outcome::Success(user) if user.role.can_manage_master() => {
                Outcome::Success(MasterGuard(user))
            }
            Outcome::Success(_) => {
                Outcome::Error((Status::Forbidden, "Memerlukan peran Admin/Ketua/Sekretaris K3"))
            }
            Outcome::Error(e) => Outcome::Error(e),
            Outcome::Forward(f) => Outcome::Forward(f),
        }
    }
}
