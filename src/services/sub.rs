use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    misc::error::{AppError, Res},
    models::sub::{SubscriptionPlan, UserSubscription},
    repo,
};

pub async fn get_all_sub_plans(pool: &PgPool) -> Res<Vec<SubscriptionPlan>> {
    repo::sub::get_all_sub_plans(pool)
        .await
        .map_err(AppError::from)
}

pub async fn get_sub_plan_by_id(pool: &PgPool, id: &Uuid) -> Res<SubscriptionPlan> {
    repo::sub::get_sub_plan_by_id(pool, id)
        .await
        .map_err(AppError::from)
}

pub async fn create_user_sub(
    pool: &PgPool,
    user_id: &Uuid,
    plan_id: &Uuid,
) -> Res<UserSubscription> {
    let sub_exists = repo::sub::exists_active_user_sub_with_plan(pool, user_id, plan_id).await?;
    if sub_exists {
        return Err(AppError::Conflict(
            "User already has an active subscription to this plan".to_string(),
        ));
    }
    repo::sub::update_user_sub_to_status_cancel(pool, user_id).await?;
    let sub = repo::sub::insert_user_subscription(pool, user_id, plan_id).await?;
    Ok(sub)
}

pub async fn get_user_sub(pool: &PgPool, user_id: &Uuid) -> Res<UserSubscription> {
    repo::sub::get_user_sub(pool, user_id)
        .await
        .map_err(AppError::from)
}
