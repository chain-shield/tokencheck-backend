use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportFilter {
    pub method: Option<String>,
    pub status_code: Option<i32>,
    pub path: Option<String>,
}
