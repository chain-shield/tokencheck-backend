use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::types::JsonValue;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow, Serialize, ToSchema)]
pub struct SubscriptionPlan {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub price: Option<f32>,
    pub daily_api_limit: Option<i32>,
    pub monthly_api_limit: Option<i32>,
    pub features: Option<JsonValue>,
    pub active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize, ToSchema)]
pub struct UserSubscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub plan_id: Uuid,
    pub start_date: NaiveDateTime,
    pub end_date: Option<NaiveDateTime>,
    pub status: String,
    pub auto_renew: bool,
    pub custom_api_limit: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
