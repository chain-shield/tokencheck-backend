use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize, ToSchema)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub company_name: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub verification_origin: String,
    pub verified: bool,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct AuthCredentials {
    pub user_id: Uuid,
    pub password_hash: String,
}
