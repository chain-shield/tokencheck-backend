use actix_web::{HttpResponse, Responder, get, post, web};
use oauth2::{AuthorizationCode, CsrfToken, Scope, TokenResponse};
use sqlx::PgPool;
use std::sync::Arc;

use crate::server::{
    config::Config,
    dtos::{
        auth::{AuthResponse, LoginRequest, RegisterRequest},
        oauth::OAuthCallbackQuery,
    },
    misc::{
        error::{AppError, Res},
        oauth::OAuthProvider,
        response::Success,
    },
    models::user::User,
    services,
};

#[utoipa::path(
    post,
    path = "/api/auth/register",
    tag = "Authentication",
    summary = "Register new user",
    description = "Creates a new user account with the provided credentials.",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = User),
        (status = 400, description = "Username already exists"),
        (status = 500, description = "Failed to register user")
    )
)]
#[post("/register")]
async fn register(
    req: web::Json<RegisterRequest>,
    pool: web::Data<Arc<sqlx::PgPool>>,
) -> impl Responder {
    let pg_pool: &PgPool = &**pool;
    let username_exists = services::user::exists_user_by_email(pg_pool, req.email.clone()).await?;
    if username_exists {
        return Err(AppError::BadRequest("Username already exists".to_string()));
    }
    let user = services::user::create_user_with_credentials(pg_pool, &req.into_inner()).await?;
    Ok(Success::created(user))
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "Authentication",
    summary = "User login",
    description = "Authenticates a user and returns an access token.",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 500, description = "Login failed")
    )
)]
#[post("/login")]
pub async fn login(
    login_data: web::Json<LoginRequest>,
    config: web::Data<Arc<Config>>,
    pool: web::Data<Arc<PgPool>>,
) -> Result<impl Responder, crate::server::misc::error::AppError> {
    let pg_pool: &PgPool = &**pool;
    let user = services::auth::authenticate_user(pg_pool, &login_data.into_inner()).await?;
    let token = services::auth::generate_jwt(&user.id.to_string(), &config.jwt_config)?;
    Success::ok(AuthResponse { token, user })
}

#[utoipa::path(
    get,
    path = "/api/auth/oauth/{provider}",
    tag = "Authentication",
    summary = "OAuth provider authentication",
    description = "Redirects to an OAuth provider's authentication page.",
    params(
        ("provider" = String, Path, description = "OAuth provider name (google, github, etc.)")
    ),
    responses(
        (status = 302, description = "Redirect to OAuth provider"),
        (status = 400, description = "Invalid provider")
    )
)]
#[get("oauth/{provider}")]
pub async fn auth_provider(
    path: web::Path<String>,
    config: web::Data<Arc<Config>>,
) -> Res<impl Responder> {
    let provider = OAuthProvider::from_str(path.as_str())?;
    let client = services::auth::create_oauth_client(&provider, &config);

    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scopes(
            provider
                .get_scopes()
                .into_iter()
                .map(|s| Scope::new(s.to_string())),
        )
        .url();

    Ok(HttpResponse::Found()
        .append_header(("Location", auth_url.to_string()))
        .finish())
}

#[utoipa::path(
    get,
    path = "/api/auth/oauth/{provider}/callback",
    tag = "Authentication",
    summary = "OAuth callback handling",
    description = "Processes OAuth provider callback and creates or authenticates a user.",
    params(
        ("provider" = String, Path, description = "OAuth provider name (google, github, etc.)"),
        ("code" = String, Query, description = "Authorization code from OAuth provider")
    ),
    responses(
        (status = 200, description = "Authentication successful", body = AuthResponse),
        (status = 400, description = "Invalid provider"),
        (status = 500, description = "Authentication failed")
    )
)]
#[get("oauth/{provider}/callback")]
async fn auth_provider_callback(
    path: web::Path<String>,
    query: web::Query<OAuthCallbackQuery>,
    config: web::Data<Arc<Config>>,
    pool: web::Data<Arc<PgPool>>,
) -> impl Responder {
    let provider = OAuthProvider::from_str(path.as_str())?;
    let client = services::auth::create_oauth_client(&provider, &config);
    let pg_pool: &PgPool = &**pool;

    let http_client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    let token = client
        .exchange_code(AuthorizationCode::new(query.code.clone()))
        .request_async(&http_client)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to exchange code. {}", e)))?;

    let access_token = token.access_token().secret();
    let user_data = services::auth::fetch_provider_user_data(&provider, access_token).await?;

    let existing_user =
        services::user::exists_user_by_email(pg_pool, user_data.email.clone()).await?;

    if existing_user {
        let user = services::user::get_user_by_email(pg_pool, user_data.email).await?;
        let token = services::auth::generate_jwt(&user.id.to_string(), &config.jwt_config)?;
        Success::ok(AuthResponse { token, user })
    } else {
        let user = services::user::create_user_with_oauth(pg_pool, &user_data, &provider).await?;
        let token = services::auth::generate_jwt(&user.id.to_string(), &config.jwt_config)?;
        Success::ok(AuthResponse { token, user })
    }
}
