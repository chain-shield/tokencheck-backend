use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{SubscriptionTier, UserSubscription};

pub async fn get_all_subscription_tiers(
    pool: &PgPool,
) -> Result<Vec<SubscriptionTier>, sqlx::Error> {
    sqlx::query_as!(
        SubscriptionTier,
        r#"
        SELECT 
            id, 
            name, 
            daily_limit, 
            monthly_limit, 
            rate_limit_per_second, 
            features, 
            price_monthly
        FROM subscription_tiers
        ORDER BY price_monthly ASC
        "#
    )
    .fetch_all(pool)
    .await
}

pub async fn get_subscription_tier_by_id(
    pool: &PgPool,
    id: i32,
) -> Result<Option<SubscriptionTier>, sqlx::Error> {
    sqlx::query_as!(
        SubscriptionTier,
        r#"
        SELECT 
            id, 
            name, 
            daily_limit, 
            monthly_limit, 
            rate_limit_per_second, 
            features, 
            price_monthly
        FROM subscription_tiers
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn create_user_subscription(
    pool: &PgPool,
    user_id: Uuid,
    tier_id: i32,
) -> Result<UserSubscription, sqlx::Error> {
    let tier_exists = sqlx::query!(
        "SELECT COUNT(*) FROM subscription_tiers WHERE id = $1",
        tier_id
    )
    .fetch_one(pool)
    .await?
    .count
    .unwrap_or(0)
        > 0;

    if !tier_exists {
        return Err(sqlx::Error::RowNotFound);
    }

    sqlx::query!(
        "UPDATE user_subscriptions 
         SET subscription_status = 'canceled'
         WHERE user_id = $1 AND subscription_status = 'active'",
        user_id
    )
    .execute(pool)
    .await?;

    let subscription = sqlx::query_as!(
        UserSubscription,
        r#"
        INSERT INTO user_subscriptions (user_id, tier_id, start_date, subscription_status)
        VALUES ($1, $2, CURRENT_TIMESTAMP, 'active')
        RETURNING user_id, tier_id, start_date, subscription_status
        "#,
        user_id,
        tier_id
    )
    .fetch_one(pool)
    .await?;

    Ok(subscription)
}

pub async fn get_user_subscription(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<UserSubscription>, sqlx::Error> {
    sqlx::query_as!(
        UserSubscription,
        r#"
        SELECT user_id, tier_id, start_date, subscription_status
        FROM user_subscriptions
        WHERE user_id = $1 AND subscription_status = 'active'
        ORDER BY start_date DESC
        LIMIT 1
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await
}
