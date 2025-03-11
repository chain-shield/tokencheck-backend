use sqlx::PgPool;

use crate::models::LogEntry;

type Result<T> = std::result::Result<T, sqlx::Error>;

pub async fn get_report(pool: &PgPool, method: Option<String>, code: Option<i32>, path: Option<String>) -> Result<Vec<LogEntry>> {
    let mut query_conditions = Vec::new();
    let mut params = Vec::new();
    let mut param_count = 1;

    let mut query_base = "SELECT * FROM request_logs".to_string();

    if let Some(method) = method {
        query_conditions.push(format!("method = ${}", param_count));
        params.push(method.clone());
        param_count += 1;
    }
    if let Some(status_code) = code {
        query_conditions.push(format!("status_code = ${}::INTEGER", param_count));
        params.push(status_code.to_string());
        param_count += 1;
    }
    if let Some(path) = path {
        query_conditions.push(format!("path = ${}", param_count));
        params.push(path.clone());
    }

    if !query_conditions.is_empty() {
        query_base.push_str(" WHERE ");
        query_base.push_str(&query_conditions.join(" AND "));
    }

    let mut query = sqlx::query_as::<_, LogEntry>(&query_base);

    for param in params {
        query = query.bind(param);
    }

    query.fetch_all(pool).await
}