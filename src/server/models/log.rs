use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::types::{ipnetwork::IpNetwork, JsonValue};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow, Serialize, ToSchema)]
pub struct Log {
    pub id: Uuid,
    pub timestamp: NaiveDateTime,
    pub method: String,
    pub path: String,
    pub status_code: i32,
    pub user_id: Option<Uuid>,
    pub params: Option<JsonValue>,
    pub request_body: Option<JsonValue>,
    pub response_body: Option<JsonValue>,
    #[schema(value_type = String)]
    pub ip_address: IpNetwork,
    pub user_agent: String,
}
