use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

#[derive(Deserialize)]
pub struct ReportFilter {
    pub method: Option<String>,
    pub status_code: Option<i32>,
    pub path: Option<String>,
}

#[derive(sqlx::FromRow, Serialize)]
pub struct LogEntry {
    pub id: i32,
    pub timestamp: NaiveDateTime,
    pub method: String,
    pub path: String,
    pub status_code: i32,
    pub params: Option<serde_json::Value>,
    pub payload: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct Item {
    pub id: i32,
    pub name: String,
}