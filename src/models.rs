use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,                         // subject (user id)
    pub exp: usize,                          // expiration time
    pub user_id: Uuid,                       // explicit user ID for convenience
    pub tier_id: Option<i32>,                // subscription tier ID
    pub subscription_status: Option<String>, // status of subscription
}

#[derive(Serialize, Deserialize)]
pub struct Item {
    pub id: Uuid,
    pub name: String,
}

#[derive(Deserialize)]
pub struct CreateItemRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportFilter {
    pub method: Option<String>,
    pub status_code: Option<i32>,
    pub path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct LogEntry {
    pub id: i32,
    pub timestamp: NaiveDateTime,
    pub method: String,
    pub path: String,
    pub status_code: i32,
    pub params: Option<serde_json::Value>,
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionTier {
    pub id: i32,
    pub name: String,
    pub daily_limit: i32,
    pub monthly_limit: i32,
    pub rate_limit_per_second: i32,
    pub features: String,
    pub price_monthly: BigDecimal,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct UserSubscription {
    pub user_id: Uuid,
    pub tier_id: i32,
    pub start_date: chrono::NaiveDateTime,
    pub subscription_status: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub tier_id: i32,
}
