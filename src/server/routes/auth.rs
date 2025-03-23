use actix_session::Session;
use actix_web::{HttpResponse, Responder, get, http::header::LOCATION, post, web};
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

/// Registers a new user with the provided credentials.
///
/// This endpoint creates a new user account in the system. It first checks if a user
/// with the provided email already exists, and if not, creates a new user record.
///
/// # Errors
/// - Returns a 400 error if a user with the provided email already exists
/// - Returns a 500 error if user creation fails
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

/// Authenticates a user with email and password credentials.
///
/// This endpoint validates the provided credentials and, if successful,
/// generates a JWT token for the authenticated user.
///
/// # Errors
/// - Returns a 401 error if credentials are invalid
/// - Returns a 500 error if authentication fails for other reasons
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

/// Initiates OAuth authentication flow with the specified provider.
///
/// This endpoint redirects the user to the OAuth provider's authentication page.
/// The provider is specified in the URL path.
///
/// # Errors
/// - Returns a 400 error if the provider is invalid or not supported
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

/// Handles the OAuth callback from the provider.
///
/// This endpoint processes the callback from the OAuth provider after user authentication.
/// It exchanges the authorization code for an access token, fetches user data from the provider,
/// and either creates a new user account or authenticates an existing user.
///
/// After successful authentication, it stores the user data and token in the session
/// and redirects to the web application's callback URL.
///
/// # Errors
/// - Returns a 400 error if the provider is invalid
/// - Returns a 500 error if authentication fails for any reason
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
    session: Session,
) -> Res<impl Responder> {
    let provider = OAuthProvider::from_str(path.as_str())?;
    log::info!(
        "OAuth callback processing for provider: {}",
        provider.as_str()
    );

    let client = services::auth::create_oauth_client(&provider, &config);
    let pg_pool: &PgPool = &**pool;

    let http_client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to build HTTP client: {}", e)))?;

    log::info!("Exchanging authorization code for access token");
    let token = client
        .exchange_code(AuthorizationCode::new(query.code.clone()))
        .request_async(&http_client)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to exchange code. {}", e)))?;

    let access_token = token.access_token().secret();
    log::info!("Successfully obtained access token, fetching user data");

    let user_data = services::auth::fetch_provider_user_data(&provider, access_token).await?;
    log::info!("User data retrieved: {:?}", user_data);

    let existing_user =
        services::user::exists_user_by_email(pg_pool, user_data.email.clone()).await?;
    log::info!("User exists check: {}", existing_user);

    let auth_response = if existing_user {
        log::info!("Authenticating existing user");
        let user = services::user::get_user_by_email(pg_pool, user_data.email).await?;
        let token = services::auth::generate_jwt(&user.id.to_string(), &config.jwt_config)?;
        AuthResponse { token, user }
    } else {
        log::info!("Creating new user with OAuth data");
        let user = services::user::create_user_with_oauth(pg_pool, &user_data, &provider).await?;
        let token = services::auth::generate_jwt(&user.id.to_string(), &config.jwt_config)?;
        AuthResponse { token, user }
    };

    log::info!("Authentication successful, preparing to set session data");

    let user_string = serde_json::to_string(&auth_response.user)
        .map_err(|e| AppError::Internal(format!("Failed to serialize user: {}", e)))?;

    // append provider to redirect_uri
    let redirect_uri = format!(
        "{}{}",
        config.web_app_auth_callback_url.as_str(),
        provider.as_str()
    );
    log::info!("Redirecting to: {}", redirect_uri);

    // Try to insert session data
    match session.insert("token", &auth_response.token) {
        Ok(_) => log::info!("Token inserted into session successfully"),
        Err(e) => log::error!("Failed to insert token: {:?}", e),
    }

    match session.insert("user", &user_string) {
        Ok(_) => log::info!("User data inserted into session successfully"),
        Err(e) => log::error!("Failed to insert user data: {:?}", e),
    }

    // We still need to return errors if session insertion fails
    session
        .insert("token", &auth_response.token)
        .map_err(|_| AppError::Internal("Failed to insert token cookie".to_string()))?;
    session
        .insert("user", &user_string)
        .map_err(|_| AppError::Internal("Failed to insert user cookie".to_string()))?;

    Ok(HttpResponse::Found()
        .append_header((LOCATION, redirect_uri))
        .finish())
}
