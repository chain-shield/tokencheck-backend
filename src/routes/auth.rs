use actix_web::{post, web, HttpResponse, Responder};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;

use crate::{
    middlewares::{self, auth::{generate_jwt, register_user, JwtConfig}},
    models::{LoginRequest, LoginResponse, RegisterRequest},
    services,
};

#[post("/register")]
async fn register(
    req: web::Json<RegisterRequest>,
    pool: web::Data<Arc<sqlx::PgPool>>,
    jwt_config: web::Data<JwtConfig>,
) -> impl Responder {
    let pg_pool: &PgPool = &**pool;

    let username_exists = services::user::exists_user_by_name(pg_pool, req.username.clone())
        .await
        .unwrap_or(false);

    if username_exists {
        return HttpResponse::BadRequest().json(json!({
            "error": "Username already exists"
        }));
    }

    match register_user(pg_pool, req.into_inner()).await {
        Ok(user) => match generate_jwt(&user.id.to_string(), &jwt_config) {
            Ok(token) => HttpResponse::Created().json(LoginResponse { token, user }),
            Err(_) => HttpResponse::InternalServerError().json(json!({
                "error": "Failed to generate token"
            })),
        },
        Err(_) => HttpResponse::InternalServerError().json(json!({
            "error": "Failed to register user"
        })),
    }
}

#[post("/login")]
pub async fn login(
    login_data: web::Json<LoginRequest>,
    jwt_config: web::Data<JwtConfig>,
    pool: web::Data<Arc<PgPool>>,
) -> impl Responder {
    let pg_pool: &PgPool = &**pool;
    match middlewares::auth::authenticate_user(pg_pool, login_data.into_inner()).await {
        Ok(Some(user)) => {
            match middlewares::auth::update_jwt_with_subscription(
                &user.id.to_string(),
                pg_pool,
                &jwt_config,
            )
            .await
            {
                Ok(token) => HttpResponse::Ok().json(LoginResponse { token, user }),
                Err(_) => HttpResponse::InternalServerError().body("Failed to generate token"),
            }
        }
        Ok(None) => HttpResponse::Unauthorized().body("Invalid credentials"),
        Err(_) => HttpResponse::InternalServerError().body("Login failed"),
    }
}
