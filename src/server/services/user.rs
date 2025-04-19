use crate::env_config::Config;
use crate::server;
use crate::server::dtos::auth::RegisterRequest;
use crate::server::dtos::oauth::OAuthUserData;
use crate::server::dtos::user::{AuthProviderCreateRequest, UserCreateRequest};
use crate::server::misc::error::Res;
use crate::server::misc::oauth::OAuthProvider;
use crate::server::misc::user::UserVerificationOrigin;
use crate::server::models::user::AuthCredentials;
use crate::server::{models::user::User, repo};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{password_hash::PasswordHasher, Argon2};

use sqlx::PgPool;
use uuid::Uuid;

use super::subscription_client::{self, SubscriptionClient};

pub async fn exists_user_by_email(pool: &PgPool, email: String) -> Res<bool> {
    repo::user::exists_user_by_email(pool, email).await
}
pub async fn get_user_by_email(pool: &PgPool, email: String) -> Res<User> {
    repo::user::get_user_by_email(pool, email).await
}
pub async fn get_user_by_id(pool: &PgPool, user_id: Uuid) -> Res<User> {
    repo::user::get_user_by_id(pool, user_id).await
}

/// Inserts user record and OAuth data to the database.
/// Used when signing in using OAuth provider.
pub async fn create_user_with_oauth(
    pool: &PgPool,
    user_data: &OAuthUserData,
    provider: &OAuthProvider,
    config: &Config,
) -> Res<User> {
    let mut tx = pool.begin().await?;

    // create Stripe customer
    let subscription_client =
        SubscriptionClient::new(config.subs_service_url.clone(), config.subs_api_key.clone());
    let stripe_customer = subscription_client
        .create_stripe_customers(
            &user_data.first_name,
            &user_data.last_name,
            &user_data.email,
        )
        .await?;

    log::info!(
        "Stripe customer created => {:?}",
        stripe_customer.customer_id
    );
    // insert user
    let user = server::repo::user::insert_user(
        &mut *tx,
        UserCreateRequest {
            email: user_data.email.clone(),
            first_name: user_data.first_name.clone(),
            last_name: user_data.last_name.clone(),
            company_name: None,
            verification_origin: UserVerificationOrigin::OAuth,
            stripe_customer_id: Some(stripe_customer.customer_id.to_string()),
        },
    )
    .await?;

    // insert provider's user data
    server::repo::user::insert_user_with_provider(
        &mut *tx,
        AuthProviderCreateRequest {
            user_id: user.id,
            provider: provider.as_str().to_string(),
            provider_user_id: user_data.provider_user_id.clone(),
        },
    )
    .await?;

    tx.commit().await?;
    Ok(user)
}

/// Inserts user record and credentials to the database.
/// User when signing in using credentials.
pub async fn create_user_with_credentials(
    pool: &PgPool,
    req: &RegisterRequest,
    config: &Config,
) -> Res<User> {
    let mut tx = pool.begin().await?;

    // create Stripe customer
    let subscription_client =
        SubscriptionClient::new(config.subs_service_url.clone(), config.subs_api_key.clone());
    let stripe_customer = subscription_client
        .create_stripe_customers(&req.first_name, &req.last_name, &req.email)
        .await?;

    // insert user
    let user = server::repo::user::insert_user(
        &mut *tx,
        UserCreateRequest {
            email: req.email.clone(),
            first_name: req.first_name.clone(),
            last_name: req.last_name.clone(),
            company_name: req.company_name.clone(),
            verification_origin: UserVerificationOrigin::Email,
            stripe_customer_id: Some(stripe_customer.customer_id.to_string()),
        },
    )
    .await?;

    // hash the password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(req.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    // insert credentials
    server::repo::user::insert_user_with_credentials(
        &mut *tx,
        AuthCredentials {
            user_id: user.id,
            password_hash,
        },
    )
    .await?;

    tx.commit().await?;
    Ok(user)
}
