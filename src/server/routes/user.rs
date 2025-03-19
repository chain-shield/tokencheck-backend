use std::sync::Arc;

use actix_web::{Responder, get, web};
use sqlx::PgPool;

use crate::server::{
    misc::response::Success,
    models::{auth::Claims, user::User},
    services,
};

#[utoipa::path(
    get,
    path = "api/secured/me",
    tag = "User",
    summary = "Get current user information",
    description = "Retrieves information about the authenticated user.",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "User information retrieved successfully", body = User),
        (status = 500, description = "Failed to get user information")
    )
)]
#[get("/me")]
async fn me(claims: web::ReqData<Claims>, pool: web::Data<Arc<sqlx::PgPool>>) -> impl Responder {
    let user_id = claims.user_id;
    let pg_pool: &PgPool = &**pool;
    let user = services::user::get_user_by_id(pg_pool, user_id).await?;
    Success::ok(user)
}
