use crate::server::{
    dtos::user::{AuthProviderCreateRequest, UserCreateRequest},
    misc::{
        error::{AppError, Res},
        user::UserVerificationOrigin,
    },
    models::user::{AuthCredentials, User},
};
use sqlx::{Executor, Postgres};
use uuid::Uuid;

pub async fn exists_user_by_email<'e, E: Executor<'e, Database = Postgres>>(
    executor: E,
    email: String,
) -> Res<bool> {
    sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1) as exists",
        email
    )
    .fetch_one(executor)
    .await
    .map(|row| row.exists.unwrap_or(false))
    .map_err(AppError::from)
}
pub async fn get_user_by_email<'e, E: Executor<'e, Database = Postgres>>(
    executor: E,
    email: String,
) -> Res<User> {
    sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", email)
        .fetch_one(executor)
        .await
        .map_err(AppError::from)
}
pub async fn get_user_by_id<'e, E: Executor<'e, Database = Postgres>>(
    executor: E,
    user_id: Uuid,
) -> Res<User> {
    sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
        .fetch_one(executor)
        .await
        .map_err(AppError::from)
}

pub async fn insert_user<'e, E: Executor<'e, Database = Postgres>>(
    executor: E,
    data: UserCreateRequest,
) -> Res<User> {
    let verified = data.verification_origin == UserVerificationOrigin::OAuth;
    sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (email, first_name, last_name, company_name, stripe_customer_id, verification_origin, verified)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#,
        data.email,
        data.first_name,
        data.last_name,
        data.company_name,
        data.stripe_customer_id,
        data.verification_origin.to_string(),
        verified
    )
    .fetch_one(executor)
    .await
    .map_err(AppError::from)
}

pub async fn insert_user_with_provider<'e, E: Executor<'e, Database = Postgres>>(
    executor: E,
    data: AuthProviderCreateRequest,
) -> Res<()> {
    sqlx::query!(
        r#"
        INSERT INTO auth_providers (user_id, provider, provider_user_id)
        VALUES ($1, $2, $3)
        "#,
        data.user_id,
        data.provider,
        data.provider_user_id
    )
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn insert_user_with_credentials<'e, E: Executor<'e, Database = Postgres>>(
    executor: E,
    data: AuthCredentials,
) -> Res<()> {
    sqlx::query!(
        r#"
        INSERT INTO auth_credentials (user_id, password_hash)
        VALUES ($1, $2)
        "#,
        data.user_id,
        data.password_hash
    )
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn get_user_with_password_hash<'e, E: Executor<'e, Database = Postgres>>(
    executor: E,
    email: String,
) -> Res<(User, AuthCredentials)> {
    sqlx::query!(
        r#"
        SELECT u.*, ac.password_hash
        FROM users u
        JOIN auth_credentials ac ON u.id = ac.user_id
        WHERE u.email = $1
        "#,
        email
    )
    .fetch_one(executor)
    .await
    .map(|record| {
        (
            User {
                id: record.id,
                email: record.email,
                first_name: record.first_name,
                last_name: record.last_name,
                company_name: record.company_name,
                created_at: record.created_at,
                updated_at: record.updated_at,
                stripe_customer_id: record.stripe_customer_id,
                verification_origin: record.verification_origin,
                verified: record.verified,
            },
            AuthCredentials {
                user_id: record.id,
                password_hash: record.password_hash,
            },
        )
    })
    .map_err(AppError::from)
}

//check if a new password is different than the previous one and if update is successful
pub async fn update_user_password_with_verification<E>(
    executor: &E,
    user_id: Uuid,
    old_password: String,
    new_password: String,
) -> Res<bool>
where
    for<'e> &'e E: Executor<'e, Database = Postgres>,
{
    // Fetch current password hash
    let credentials = sqlx::query!(
        "SELECT password_hash FROM auth_credentials WHERE user_id = $1",
        user_id
    )
    .fetch_optional(executor)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::from(anyhow::anyhow!("No credentials found for user")))?;

    // Verify old password
    let is_valid = bcrypt::verify(&old_password, &credentials.password_hash)
        .map_err(|e| AppError::from(anyhow::anyhow!(e)))?;
    if !is_valid {
        return Err(AppError::from(anyhow::anyhow!("Invalid old password")));
    }

    // Check if new password is different by verifying it against the current hash
    let is_same_password = bcrypt::verify(&new_password, &credentials.password_hash)
        .map_err(|e| AppError::from(anyhow::anyhow!(e)))?;
    if is_same_password {
        return Ok(false); // New password is the same as the old one
    }

    // Hash new password
    let new_password_hash = bcrypt::hash(&new_password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::from(anyhow::anyhow!(e)))?;

    // Update password hash
    let result = sqlx::query!(
        r#"
        UPDATE auth_credentials
        SET password_hash = $1
        WHERE user_id = $2
        "#,
        new_password_hash,
        user_id
    )
    .execute(executor)
    .await
    .map_err(AppError::from)?;

    // Return true if the update affected a row, false otherwise
    Ok(result.rows_affected() > 0)
}

