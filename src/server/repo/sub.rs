use crate::server::{
    misc::error::{AppError, Res},
    models::sub::{SubscriptionPlan, UserSubscription},
};
use sqlx::{Executor, Postgres};
use uuid::Uuid;

pub async fn get_all_sub_plans<'e, E: Executor<'e, Database = Postgres>>(
    executor: E,
) -> Res<Vec<SubscriptionPlan>> {
    sqlx::query_as!(
        SubscriptionPlan,
        r#"
        SELECT *
        FROM subscription_plans
        ORDER BY price ASC
        "#
    )
    .fetch_all(executor)
    .await
    .map_err(AppError::from)
}

pub async fn get_sub_plan_by_id<'e, E: Executor<'e, Database = Postgres>>(
    executor: E,
    id: &Uuid,
) -> Res<SubscriptionPlan> {
    sqlx::query_as!(
        SubscriptionPlan,
        r#"
        SELECT *
        FROM subscription_plans
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(executor)
    .await
    .map_err(AppError::from)?
    .ok_or(AppError::NotFound("Subscription plan not found".into()))
}

pub async fn exists_active_user_sub_with_plan<'e, E: Executor<'e, Database = Postgres>>(
    executor: E,
    user_id: &Uuid,
    plan_id: &Uuid,
) -> Res<bool> {
    let count = sqlx::query!(
        "SELECT COUNT(*) FROM user_subscriptions WHERE user_id = $1 AND plan_id = $2 AND status = 'active'",
        user_id,
        plan_id
    )
    .fetch_one(executor)
    .await?
    .count
    .unwrap_or(0);

    Ok(count > 0)
}

pub async fn get_user_sub<'e, E: Executor<'e, Database = Postgres>>(
    executor: E,
    user_id: &Uuid,
) -> Res<UserSubscription> {
    sqlx::query_as!(
        UserSubscription,
        r#"
        SELECT *
        FROM user_subscriptions
        WHERE user_id = $1 AND status = 'active'
        ORDER BY start_date DESC
        LIMIT 1
        "#,
        user_id
    )
    .fetch_optional(executor)
    .await
    .map_err(AppError::from)?
    .ok_or(AppError::NotFound("User subscription not found".into()))
}

pub async fn update_user_sub_to_status_cancel<'e, E: Executor<'e, Database = Postgres>>(
    executor: E,
    user_id: &Uuid,
) -> Res<()> {
    sqlx::query!(
        "UPDATE user_subscriptions 
         SET status = 'canceled', end_date = CURRENT_TIMESTAMP
         WHERE user_id = $1 AND status = 'active'",
        user_id
    )
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn insert_user_subscription<'e, E: Executor<'e, Database = Postgres>>(
    executor: E,
    user_id: &Uuid,
    plan_id: &Uuid,
) -> Res<UserSubscription> {
    let subscription = sqlx::query_as!(
        UserSubscription,
        r#"
        INSERT INTO user_subscriptions (user_id, plan_id, start_date, status, auto_renew, created_at, updated_at)
        VALUES ($1, $2, CURRENT_TIMESTAMP, 'active', TRUE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        RETURNING id, user_id, plan_id, start_date, end_date, status, auto_renew, custom_api_limit, created_at, updated_at
        "#,
        user_id,
        plan_id
    )
    .fetch_one(executor)
    .await?;
    Ok(subscription)
}
