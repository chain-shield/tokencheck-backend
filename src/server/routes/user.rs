use std::sync::Arc;

use actix_web::{get, web, Responder};
use sqlx::PgPool;

use crate::server::{
    misc::response::Success,
    models::{auth::JwtClaims, user::User},
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
async fn me(claims: web::ReqData<JwtClaims>, pool: web::Data<Arc<sqlx::PgPool>>) -> impl Responder {
    let user_id = claims.user_id;
    let pg_pool: &PgPool = &**pool;
    let user = services::user::get_user_by_id(pg_pool, user_id).await?;
    Success::ok(user)
}

#[derive(Deserialize)]
pub struct UpdatePasswordRequest {
    old_password: String,
    new_password: String,
}

#[utoipa::path(
    post,
    path = "/api/secured/update_password",
    tag = "User",
    summary = "Update user password",
    description = "Updates the authenticated user's password after verifying the old password.",
    security(
        ("bearer_auth" = [])
    ),
    request_body(
        content = UpdatePasswordRequest,
        description = "Old and new password for updating",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "Password update result", body = bool),
        (status = 400, description = "Invalid old password or same password provided"),
        (status = 500, description = "Failed to update password")
    )
)]
#[post("/update_password")]
async fn update_password(
    claims: web::ReqData<JwtClaims>,
    pool: web::Data<Arc<sqlx::PgPool>>,
    data: web::Json<UpdatePasswordRequest>,
) -> impl Responder {
    let user_id = claims.user_id;
    let pg_pool: &PgPool = &**pool;
    let updated = services::user::update_user_password_with_verification(
        pg_pool,
        user_id,
        data.old_password.clone(),
        data.new_password.clone(),
    )
    .await?;
    Success::ok(updated)
}