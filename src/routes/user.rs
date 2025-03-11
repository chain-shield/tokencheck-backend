use std::sync::Arc;

use actix_web::{get, web, HttpResponse, Responder};
use serde_json::json;
use sqlx::PgPool;

use crate::{models::Claims, services};

#[get("/me")]
async fn me(
    claims: web::ReqData<Claims>,
    pool: web::Data<Arc<sqlx::PgPool>>,
) -> impl Responder {
    let user_id = claims.user_id;
    let pg_pool: &PgPool = &**pool;
    match services::user::get_user_by_id(pg_pool, user_id).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::InternalServerError().json(json!({
            "error": "Failed to get user"
        })),
    }
}