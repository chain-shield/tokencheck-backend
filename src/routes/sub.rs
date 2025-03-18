use actix_web::{get, post, web, Responder};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    config::Config,
    dtos::sub::{SubscriptionCreateRequest, SubscriptionResponse},
    misc::response::Success,
    models::{
        auth::Claims,
        sub::{SubscriptionPlan, UserSubscription},
    },
    services::{self},
};

#[utoipa::path(
    get,
    path = "/api/secured/sub/plans",
    tag = "Subscription",
    summary = "Get all subscription plans",
    description = "Retrieves all available subscription plans.",
    responses(
        (status = 200, description = "Plans retrieved successfully", body = Vec<SubscriptionPlan>),
        (status = 500, description = "Error retrieving subscription plans")
    )
)]
#[get("/sub/plans")]
pub async fn get_all_plans(pool: web::Data<Arc<PgPool>>) -> impl Responder {
    let pg_pool: &PgPool = &**pool;
    let plans = services::sub::get_all_sub_plans(pg_pool).await?;
    Success::ok(plans)
}

#[utoipa::path(
    get,
    path = "/api/secured/sub/plan/{id}",
    tag = "Subscription",
    summary = "Get subscription plan by ID",
    description = "Retrieves a specific subscription plan by its ID.",
    params(
        ("id" = Uuid, Path, description = "Subscription plan unique identifier")
    ),
    responses(
        (status = 200, description = "Plan retrieved successfully", body = SubscriptionPlan),
        (status = 404, description = "Subscription plan not found"),
        (status = 500, description = "Database error")
    )
)]
#[get("/sub/plan/{id}")]
pub async fn get_plan(path: web::Path<Uuid>, pool: web::Data<Arc<PgPool>>) -> impl Responder {
    let uuid = path.into_inner();
    let pg_pool: &PgPool = &**pool;
    let plan = services::sub::get_sub_plan_by_id(pg_pool, &uuid).await?;
    Success::ok(plan)
}

#[utoipa::path(
    post,
    path = "/api/secured/sub/subscribe",
    tag = "Subscription",
    summary = "Subscribe to a plan",
    description = "Creates a new subscription for the authenticated user.",
    security(
        ("bearer_auth" = [])
    ),
    request_body = SubscriptionCreateRequest,
    responses(
        (status = 201, description = "Subscription created successfully", body = SubscriptionResponse),
        (status = 400, description = "Invalid subscription plan"),
        (status = 500, description = "Error creating subscription")
    )
)]
#[post("/sub/subscribe")]
pub async fn subscribe(
    claims: web::ReqData<Claims>,
    subscription: web::Json<SubscriptionCreateRequest>,
    pool: web::Data<Arc<PgPool>>,
    config: web::Data<Arc<Config>>,
) -> impl Responder {
    let user_id = claims.user_id;
    let plan_id = subscription.plan_id;
    let pg_pool: &PgPool = &**pool;

    let subscription = services::sub::create_user_sub(pg_pool, &user_id, &plan_id).await?;
    let token =
        services::auth::update_jwt_with_sub(&user_id.to_string(), pg_pool, &config.jwt_config)
            .await?;

    Success::created(SubscriptionResponse {
        subscription,
        token,
    })
}

#[utoipa::path(
    get,
    path = "/api/secured/sub/current",
    tag = "Subscription",
    summary = "Get current user subscription",
    description = "Retrieves the authenticated user's current subscription information.",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Subscription retrieved successfully", body = UserSubscription),
        (status = 404, description = "No active subscription found"),
        (status = 500, description = "Database error")
    )
)]
#[get("/sub/current")]
pub async fn get_my_plan(
    claims: web::ReqData<Claims>,
    pool: web::Data<Arc<PgPool>>,
) -> impl Responder {
    let user_id = claims.user_id;
    let pg_pool: &PgPool = &**pool;
    let subscription = services::sub::get_user_sub(pg_pool, &user_id).await?;
    Success::ok(subscription)
}
