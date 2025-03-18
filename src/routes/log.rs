use actix_web::{get, web, Responder};
use std::sync::Arc;

use crate::{dtos::log::ReportFilter, misc::response::Success, models::log::Log, services};

#[utoipa::path(
    get,
    path = "/api/secured/report",
    tag = "Logs",
    summary = "Get access log report",
    description = "Retrieves access logs with optional filtering criteria.",
    params(
        ("method" = Option<String>, Query, description = "Filter by HTTP method (GET, POST, etc.)"),
        ("status_code" = Option<i32>, Query, description = "Filter by HTTP status code"),
        ("path" = Option<String>, Query, description = "Filter by request path")
    ),
    responses(
        (status = 200, description = "Report retrieved successfully", body = Vec<Log>),
        (status = 500, description = "Failed to get report")
    )
)]
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
