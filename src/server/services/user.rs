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

pub async fn exists_user_by_email(pool: &PgPool, email: String) -> Res<bool> {
    repo::user::exists_user_by_email(pool, email).await
}
pub async fn get_user_by_email(pool: &PgPool, email: String) -> Res<User> {
    repo::user::get_user_by_email(pool, email).await
}
pub async fn get_user_by_id(pool: &PgPool, user_id: Uuid) -> Res<User> {
    repo::user::get_user_by_id(pool, user_id).await
}

pub async fn create_user_with_oauth(
    pool: &PgPool,
    user_data: &OAuthUserData,
    provider: &OAuthProvider,
) -> Res<User> {
    let mut tx = pool.begin().await?;

    let user = repo::user::insert_user(
        &mut *tx,
        UserCreateRequest {
            email: user_data.email.clone(),
            first_name: user_data.first_name.clone(),
            last_name: user_data.last_name.clone(),
            company_name: None,
            verification_origin: UserVerificationOrigin::OAuth,
        },
    )
    .await?;

    repo::user::insert_user_with_provider(
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

pub async fn create_user_with_credentials(pool: &PgPool, req: &RegisterRequest) -> Res<User> {
    let mut tx = pool.begin().await?;

    let user = repo::user::insert_user(
        &mut *tx,
        UserCreateRequest {
            email: req.email.clone(),
            first_name: req.first_name.clone(),
            last_name: req.last_name.clone(),
            company_name: req.company_name.clone(),
            verification_origin: UserVerificationOrigin::Email,
        },
    )
    .await?;

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(req.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    repo::user::insert_user_with_credentials(
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
