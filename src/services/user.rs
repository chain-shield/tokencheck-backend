use sqlx::PgPool;
use uuid::Uuid;
use crate::models::User;

type Result<T> = std::result::Result<T, sqlx::Error>;

pub async fn exists_user_by_name(pool: &PgPool, username: String) -> Result<bool> {
    sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE username = $1) as exists",
        username
    )
    .fetch_one(pool)
    .await
    .map(|row| row.exists.unwrap_or(false))
}

pub async fn get_user_by_id(pool: &PgPool, user_id: Uuid) -> Result<User> {
    sqlx::query_as!(
        User,
        "SELECT id::uuid, username, password_hash, created_at FROM users WHERE id = $1",
        user_id
    )
    .fetch_one(pool)
    .await
}

pub async fn get_user_by_name(pool: &PgPool, username: String) -> Result<User> {
    sqlx::query_as!(
        User,
        "SELECT id::uuid, username, password_hash, created_at FROM users WHERE username = $1",
        username
    )
    .fetch_one(pool)
    .await
}

pub async fn create_user(pool: &PgPool, username: String, password_hash: String) -> Result<User> {
    sqlx::query_as!(
        User,
        "INSERT INTO users (username, password_hash, created_at) VALUES ($1, $2, NOW()) RETURNING id::uuid, username, password_hash, created_at",
        username,
        password_hash
    )
    .fetch_one(pool)
    .await
}