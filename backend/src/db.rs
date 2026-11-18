use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn log_audit(
    pool: &PgPool,
    user_id: Option<Uuid>,
    username: &str,
    action: &str,
    entity_type: &str,
    entity_id: Option<&str>,
    details: Value,
) {
    let _ = sqlx::query(
        "INSERT INTO audit_logs (user_id, username, action, entity_type, entity_id, details)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(user_id)
    .bind(username)
    .bind(action)
    .bind(entity_type)
    .bind(entity_id)
    .bind(details)
    .execute(pool)
    .await;
}
