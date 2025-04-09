use actix_web::{get, HttpResponse, Responder};

#[utoipa::path(
    post,
    path = "/api/healthz",
    tag = "Health Check",
    summary = "Health check",
    description = "Check health of server",
    responses(
        (status = 200, description = "Server is up and running"),
    )
)]
#[get("/healthz")]
async fn healthz() -> impl Responder {
    HttpResponse::Ok().body("OK")
}
