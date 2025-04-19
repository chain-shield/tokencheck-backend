use crate::{
    env_config::Config,
    server::{
        dtos::auth::TokenValidationRequest, misc::response::Success, models::auth::JwtClaims,
        services,
    },
};
use actix_web::{post, web, Responder};
use log::info;
use std::sync::Arc;

/// Server to Server call to validate jwt token
///
/// This endpoint is used for validation in auth middleware by
/// other micro services
/// # Errors
/// - Returns a 401 error if token is not valid - Unauthorized
#[utoipa::path(
    post,
    path = "/api/auth/validate-token",
    tag = "Token Validation",
    summary = "validate jwt token",
    description = "validate jwt token for other micro services that need to make sure incoming calls are authenticated",
    request_body = TokenValidationRequest,
    responses(
        (status = 200, description = "Call successfully validated", body = JwtClaims),
        (status = 401, description = "Unauthorized"),
    )
)]
#[post("/validate-token")]
async fn validate_token(
    req: web::Json<TokenValidationRequest>,
    config: web::Data<Arc<Config>>,
) -> impl Responder {
    let claims = services::auth::validate_jwt(&req.token, &config.jwt_config.secret)?;
    info!(
        "Token validated successfully for user_id: {}",
        claims.user_id
    );
    Success::ok(claims)
}
