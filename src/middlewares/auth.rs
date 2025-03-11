use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use futures::future::{ok, Future, Ready};
use jsonwebtoken::{
    decode, encode, errors::Error as JwtError, DecodingKey, EncodingKey, Header, Validation,
};
use sqlx::{Error as SqlxError, PgPool};
use std::{pin::Pin, rc::Rc, sync::Arc};
use uuid::Uuid;

use crate::{
    models::{Claims, LoginRequest, RegisterRequest, User},
    services,
};

#[derive(Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration_hours: i64,
}

impl JwtConfig {
    pub fn from_env() -> Self {
        let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let expiration_hours = std::env::var("JWT_EXPIRATION_HOURS")
            .unwrap_or_else(|_| "24".to_string())
            .parse::<i64>()
            .expect("JWT_EXPIRATION_HOURS must be a valid number");

        JwtConfig {
            secret,
            expiration_hours,
        }
    }
}

pub fn generate_jwt(user_id: &str, config: &JwtConfig) -> Result<String, JwtError> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(config.expiration_hours))
        .expect("valid timestamp")
        .timestamp();

    let user_uuid = Uuid::parse_str(user_id).expect("valid UUID");

    let claims = Claims {
        sub: user_id.to_owned(),
        exp: expiration as usize,
        user_id: user_uuid,
        tier_id: None, // Will be populated after checking subscription
        subscription_status: None,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
}

pub async fn update_jwt_with_subscription(
    user_id: &str,
    pool: &PgPool,
    config: &JwtConfig,
) -> Result<String, JwtError> {
    let user_uuid = Uuid::parse_str(user_id).expect("valid UUID");

    let subscription = services::subs::get_user_subscription(pool, user_uuid)
        .await
        .ok()
        .flatten();

    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(config.expiration_hours))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_id.to_owned(),
        exp: expiration as usize,
        user_id: user_uuid,
        tier_id: subscription.as_ref().map(|s| s.tier_id),
        subscription_status: subscription.map(|s| s.subscription_status),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
}

pub fn validate_jwt(token: &str, config: &JwtConfig) -> Result<Claims, JwtError> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

pub struct AuthMiddleware {
    jwt_config: Rc<JwtConfig>,
}

impl AuthMiddleware {
    pub fn new(config: JwtConfig) -> Self {
        AuthMiddleware {
            jwt_config: Rc::new(config),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareService {
            service: Arc::new(service),
            jwt_config: self.jwt_config.clone(),
        })
    }
}

pub struct AuthMiddlewareService<S> {
    service: Arc<S>,
    jwt_config: Rc<JwtConfig>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Only require auth for paths containing "/api/secured"
        let path = req.path();
        if !path.contains("/api/secured") {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await.map(|res| res.map_into_boxed_body()) });
        }

        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|auth_value| {
                if auth_value.starts_with("Bearer ") {
                    Some(auth_value[7..].to_owned())
                } else {
                    None
                }
            });

        let jwt_config = self.jwt_config.clone();
        let srv = Arc::clone(&self.service);

        Box::pin(async move {
            if let Some(token) = auth_header {
                match validate_jwt(&token, &jwt_config) {
                    Ok(claims) => {
                        req.extensions_mut().insert(claims);
                        srv.call(req).await.map(|res| res.map_into_boxed_body())
                    }
                    Err(_) => {
                        let response = HttpResponse::Unauthorized()
                            .json(serde_json::json!({"error": "Invalid token"}))
                            .map_into_boxed_body();
                        Ok(req.into_response(response))
                    }
                }
            } else {
                let response = HttpResponse::Unauthorized()
                    .json(serde_json::json!({"error": "No authorization token provided"}))
                    .map_into_boxed_body();
                Ok(req.into_response(response))
            }
        })
    }
}

pub async fn register_user(
    pool: &PgPool,
    register_data: RegisterRequest,
) -> Result<User, SqlxError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(register_data.password.as_bytes(), &salt)
        .unwrap()
        .to_string();
    Ok(services::user::create_user(pool, register_data.username.clone(), password_hash).await?)
}

pub async fn authenticate_user(
    pool: &PgPool,
    login_data: LoginRequest,
) -> Result<Option<User>, SqlxError> {
    let user = services::user::get_user_by_name(pool, login_data.username.clone()).await?;
    let parsed_hash = PasswordHash::new(&user.password_hash).unwrap();
    let is_valid = Argon2::default()
        .verify_password(login_data.password.as_bytes(), &parsed_hash)
        .is_ok();

    if is_valid {
        return Ok(Some(user));
    }

    Ok(None)
}
