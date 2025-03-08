use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// Filter criteria for querying log reports
///
/// This structure allows filtering log entries by HTTP method,
/// status code, and request path.
#[derive(Deserialize)]
pub struct ReportFilter {
    /// Optional HTTP method to filter by (GET, POST, etc.)
    pub method: Option<String>,
    /// Optional HTTP status code to filter by (200, 404, 500, etc.)
    pub status_code: Option<i32>,
    /// Optional request path to filter by
    pub path: Option<String>,
}

/// Represents a database entry for an HTTP request log
///
/// This structure maps directly to the database schema for storing
/// HTTP request logs with their associated metadata.
#[derive(sqlx::FromRow, Serialize)]
pub struct LogEntry {
    /// Unique identifier for the log entry
    pub id: i32,
    /// When the request was received
    pub timestamp: NaiveDateTime,
    /// HTTP method used (GET, POST, PUT, DELETE, etc.)
    pub method: String,
    /// Request path
    pub path: String,
    /// HTTP response status code
    pub status_code: i32,
    /// Optional URL parameters as JSON
    pub params: Option<serde_json::Value>,
    /// Optional request body as JSON
    pub payload: Option<serde_json::Value>,
}

/// Generic item representation with an ID and name
///
/// Used for simple entity representations throughout the application.
#[derive(Serialize, Deserialize)]
pub struct Item {
    /// Unique identifier for the item
    pub id: i32,
    /// Name of the item
    pub name: String,
}
