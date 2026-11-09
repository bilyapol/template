use crate::controllers::auth::GoogleUserInfo;
use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub google_id: Option<String>,
    pub email: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Upsert via Google OAuth (matches on google_id).
pub async fn upsert_google(pool: &PgPool, info: &GoogleUserInfo) -> Result<User, AppError> {
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (google_id, email, name, avatar_url) \
         VALUES ($1, $2, $3, $4) \
         ON CONFLICT (google_id) \
         DO UPDATE SET email = $2, name = $3, avatar_url = $4, updated_at = NOW() \
         RETURNING *",
    )
    .bind(&info.id)
    .bind(&info.email)
    .bind(&info.name)
    .bind(&info.picture)
    .fetch_one(pool)
    .await?;
    Ok(user)
}

/// Upsert via email only (dev-mode login, no external provider).
pub async fn upsert_by_email(pool: &PgPool, email: &str, name: &str) -> Result<User, AppError> {
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (email, name) \
         VALUES ($1, $2) \
         ON CONFLICT (email) \
         DO UPDATE SET name = $2, updated_at = NOW() \
         RETURNING *",
    )
    .bind(email)
    .bind(name)
    .fetch_one(pool)
    .await?;
    Ok(user)
}

pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<User, AppError> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))
}
