use actix_web::{Responder, get, web};
use std::sync::Arc;

use crate::server::{dtos::log::ReportFilter, misc::response::Success, services};

/// # Access Log Report API
///
/// Retrieves a filtered list of access logs from the system.
///
/// # Input
/// - `pool`: Database connection pool
/// - `filter`: Query parameters for filtering logs:
///   - `method` (optional): Filter logs by HTTP method (e.g., "GET", "POST")
///   - `status_code` (optional): Filter logs by HTTP status code (e.g., 200, 404)
///   - `path` (optional): Filter logs by request path (e.g., "/api/users")
///
/// # Output
/// - Success: Returns a JSON array of log entries matching the filter criteria
/// - Error: Returns 500 if the server encounters an error retrieving logs
///
/// # Frontend Example
/// ```javascript
/// // Using fetch API with query parameters
/// // Build query string based on filters
/// const filters = new URLSearchParams();
/// if (methodFilter) filters.append('method', methodFilter);
/// if (statusCodeFilter) filters.append('status_code', statusCodeFilter);
/// if (pathFilter) filters.append('path', pathFilter);
///
/// const response = await fetch(`/api/secured/report?${filters.toString()}`, {
///   headers: {
///     'Authorization': `Bearer ${localStorage.getItem('authToken')}`
///   }
/// });
///
/// if (response.ok) {
///   const logs = await response.json();
///   console.log('Filtered logs:', logs);
///   // Example response:
///   // [
///   //   {
///   //     id: "uuid-1",
///   //     timestamp: "2023-05-01T12:34:56Z",
///   //     method: "POST",
///   //     path: "/api/users",
///   //     status_code: 201,
///   //     user_id: "user-uuid-1",
///   //     ip_address: "127.0.0.1",
///   //     ...
///   //   },
///   //   // More log entries...
///   // ]
/// }
/// ```
#[get("/report")]
pub async fn report(
    pool: web::Data<Arc<sqlx::PgPool>>,
    filter: web::Query<ReportFilter>,
) -> impl Responder {
    let report = services::log::get_report(
        &pool,
        filter.method.clone(),
        filter.status_code,
        filter.path.clone(),
    )
    .await?;
    Success::ok(report)
}
