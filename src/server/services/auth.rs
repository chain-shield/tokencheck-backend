use crate::{
    env_config::{Config, JwtConfig},
    server::{
        dtos::{auth::LoginRequest, oauth::OAuthUserData},
        misc::{
            error::{AppError, Res},
            oauth::OAuthProvider,
        },
        models::user::User,
        repo,
    },
};
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, DecodingKey, Validation};
use jsonwebtoken::{encode, EncodingKey, Header};
use log::warn;
use oauth2::basic::*;
use oauth2::*;
use sqlx::PgPool;
use uuid::Uuid;

use crate::server::{models::auth::Claims, services};

/// Generates a JWT token for the given user ID
///
/// # Arguments
/// * `user_id` - The user's UUID as a string
/// * `config` - JWT configuration containing secret and expiration settings
///
/// # Returns
/// A Result containing the JWT token string or an error
pub fn generate_jwt(user_id: &str, config: &JwtConfig) -> Res<String> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(config.expiration_hours))
        .ok_or_else(|| AppError::Internal("Failed to calculate token expiration".to_string()))?
        .timestamp();

    let user_uuid = Uuid::parse_str(user_id)
        .map_err(|_| AppError::BadRequest("Invalid user ID format".to_string()))?;

    let claims = Claims {
        user_id: user_uuid,
        exp: expiration as usize,
        plan_id: Uuid::nil(),
        sub_status: "none".to_owned(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
    .map_err(AppError::from)
}

/// Updates a JWT token with subscription information for the given user
///
/// # Arguments
/// * `user_id` - The user's UUID as a string
/// * `pool` - Database connection pool
/// * `config` - JWT configuration containing secret and expiration settings
///
/// # Returns
/// A Result containing the updated JWT token string or an error
pub async fn update_jwt_with_sub(user_id: &str, pool: &PgPool, config: &JwtConfig) -> Res<String> {
    let user_uuid = Uuid::parse_str(user_id)
        .map_err(|_| AppError::BadRequest("Invalid user ID format".to_string()))?;

    let subscription = services::sub::get_user_sub(pool, &user_uuid).await.ok();

    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(config.expiration_hours))
        .ok_or_else(|| AppError::Internal("Failed to calculate token expiration".to_string()))?
        .timestamp();

    let claims = Claims {
        user_id: user_uuid,
        exp: expiration as usize,
        plan_id: subscription.as_ref().map_or(Uuid::nil(), |sub| sub.plan_id),
        sub_status: subscription.map_or("none".to_owned(), |sub| sub.status),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
    .map_err(AppError::from)
}

/// Validates a JWT token and extracts the claims
///
/// # Arguments
/// * `token` - The JWT token string to validate
/// * `secret` - The secret key used to sign the token
///
/// # Returns
/// A Result containing the validated Claims or an error
pub fn validate_jwt(token: &str, secret: &str) -> Res<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

/// Creates an OAuth client for the specified provider
///
/// # Arguments
/// * `provider` - The OAuth provider to create a client for
/// * `config` - Application configuration containing OAuth client settings
///
/// # Returns
/// A configured OAuth client for the specified provider
pub fn create_oauth_client(
    provider: &OAuthProvider,
    config: &Config,
) -> Client<
    StandardErrorResponse<BasicErrorResponseType>,
    StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>,
    StandardRevocableToken,
    StandardErrorResponse<RevocationErrorResponseType>,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet,
> {
    let provider_client = match provider {
        OAuthProvider::GitHub => &config.github_client,
        OAuthProvider::Google => &config.google_client,
        OAuthProvider::Facebook => &config.facebook_client,
        OAuthProvider::Apple => &config.apple_client,
        OAuthProvider::Twitter => &config.twitter_client,
        // _ => panic!("Unsupported OAuth provider"),
    };

    let client_id = ClientId::new(provider_client.client_id.clone());
    let client_secret = ClientSecret::new(provider_client.client_secret.clone());
    let auth_url = AuthUrl::new(provider_client.auth_url.clone())
        .map_err(|_| AppError::Internal("Invalid authorization endpoint URL".to_string()))
        .unwrap(); // This is still unwrapped as it's a configuration error that should fail fast
    let token_url = TokenUrl::new(provider_client.token_url.clone())
        .map_err(|_| AppError::Internal("Invalid token endpoint URL".to_string()))
        .unwrap(); // This is still unwrapped as it's a configuration error that should fail fast

    let client = BasicClient::new(client_id)
        .set_client_secret(client_secret)
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(
            RedirectUrl::new(provider_client.redirect_uri.to_string())
                .map_err(|_| AppError::Internal("Invalid redirect URL".to_string()))
                .unwrap(), // This is still unwrapped as it's a configuration error that should fail fast
        );

    client
}

/// Authenticates a user with email and password
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `login_data` - Login request containing email and password
///
/// # Returns
/// A Result containing the authenticated User or an error
pub async fn authenticate_user(pool: &PgPool, login_data: &LoginRequest) -> Res<User> {
    let (user, credentials) =
        repo::user::get_user_with_password_hash(pool, login_data.email.clone())
            .await
            .map_err(|_| AppError::BadRequest("User with this email does not exist".to_string()))?;

    let parsed_hash = PasswordHash::new(&credentials.password_hash)
        .map_err(|_| AppError::Internal("Failed to parse password hash".to_string()))?;

    let is_valid = Argon2::default()
        .verify_password(login_data.password.as_bytes(), &parsed_hash)
        .is_ok();

    if is_valid {
        Ok(user)
    } else {
        Err(AppError::Unauthorized("Invalid credentials".to_string()))
    }
}

/// Fetches user data from the specified OAuth provider
///
/// # Arguments
/// * `provider` - The OAuth provider to fetch user data from
/// * `access_token` - The OAuth access token
///
/// # Returns
/// A Result containing the user data or an error
pub async fn fetch_provider_user_data(
    provider: &OAuthProvider,
    access_token: &str,
) -> Res<OAuthUserData> {
    match provider {
        OAuthProvider::GitHub => fetch_github_user_data(access_token).await,
        OAuthProvider::Google => fetch_google_user_data(access_token).await,
        prov => Err(AppError::Internal(format!(
            "Unsupported OAuth provider: {:?}",
            prov
        ))),
    }
}

/// Fetches user data from GitHub using the provided access token
///
/// # Arguments
/// * `access_token` - GitHub OAuth access token
///
/// # Returns
/// A Result containing the user data or an error
async fn fetch_github_user_data(access_token: &str) -> Res<OAuthUserData> {
    let client = reqwest::Client::new();
    let request = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "WebServer");

    let response = request
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch GitHub user data: {}", e)))?;

    if response.status().is_success() {
        let github_user: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse GitHub user data: {}", e)))?;

        let email = match github_user["email"].as_str() {
            Some(email) => email.to_string(),
            None => {
                // If the email is not public, fetch it from the emails API
                let emails_url = "https://api.github.com/user/emails";
                let emails_response = client
                    .get(emails_url)
                    .header("Authorization", format!("Bearer {}", access_token))
                    .header("User-Agent", "WebServer")
                    .send()
                    .await
                    .map_err(|e| {
                        AppError::Internal(format!("Failed to fetch GitHub emails: {}", e))
                    })?;

                if emails_response.status().is_success() {
                    let emails: Vec<serde_json::Value> =
                        emails_response.json().await.map_err(|e| {
                            AppError::Internal(format!("Failed to parse GitHub emails: {}", e))
                        })?;

                    // Find the primary email
                    emails
                        .iter()
                        .find(|email| email["primary"].as_bool().unwrap_or(false))
                        .and_then(|email| email["email"].as_str())
                        .unwrap_or("")
                        .to_string()
                } else {
                    warn!(
                        "Failed to fetch GitHub emails: {:?}",
                        emails_response.status()
                    );
                    "".to_string()
                }
            }
        };
        let name = github_user["name"].as_str().unwrap_or("").to_string();
        let names: Vec<&str> = name.split(' ').collect();
        let first_name = names.first().unwrap_or(&"").to_string();
        let last_name = names.get(1).unwrap_or(&"").to_string();
        let provider_user_id = github_user["id"].to_string();

        Ok(OAuthUserData {
            email,
            first_name,
            last_name,
            provider_user_id,
        })
    } else {
        Err(AppError::Internal(format!(
            "GitHub API returned error status: {}",
            response.status()
        )))
    }
}

/// Fetches user data from Google using the provided access token
///
/// # Arguments
/// * `access_token` - Google OAuth access token
///
/// # Returns
/// A Result containing the user data or an error
async fn fetch_google_user_data(access_token: &str) -> Res<OAuthUserData> {
    let client = reqwest::Client::new();
    let request = client
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .header("Authorization", format!("Bearer {}", access_token));

    let response = request
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch Google user data: {}", e)))?;

    if response.status().is_success() {
        let google_user: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse Google user data: {}", e)))?;

        let email = google_user["email"].as_str().unwrap_or("").to_string();
        let first_name = google_user["given_name"].as_str().unwrap_or("").to_string();
        let last_name = google_user["family_name"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let provider_user_id = google_user["sub"].to_string();

        Ok(OAuthUserData {
            email,
            first_name,
            last_name,
            provider_user_id,
        })
    } else {
        Err(AppError::Internal(format!(
            "Google API returned error status: {}",
            response.status()
        )))
    }
}
