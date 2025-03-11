use std::sync::Arc;
use actix_web::{get, web, HttpResponse, Responder};

use crate::{models::ReportFilter, services};

#[get("/report")]
pub async fn report(
    pool: web::Data<Arc<sqlx::PgPool>>,
    filter: web::Query<ReportFilter>,
) -> impl Responder {
    services::report::get_report(
        &pool,
        filter.method.clone(),
        filter.status_code,
        filter.path.clone(),
    )
    .await
    .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to get report"))
    .map(|report| HttpResponse::Ok().json(report))
}