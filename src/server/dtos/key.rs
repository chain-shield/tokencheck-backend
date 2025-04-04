use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::types::JsonValue;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct CreateApiKeyResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key: String,
    pub name: String,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub permissions: JsonValue,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiKeyDto {
    pub id: Uuid,
    pub key: String,
    pub name: String,
    pub status: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateApiKeyRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GetApiKeysRequest {
    pub name: String,
}
