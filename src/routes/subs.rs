use actix_web::{get, post, web, HttpResponse, Responder};
use sqlx::PgPool;
use std::sync::Arc;

use crate::{
    middlewares::{self, auth::JwtConfig}, models::{Claims, CreateSubscriptionRequest}, services
};

#[get("/sub/plans")]
pub async fn get_all_plans(pool: web::Data<Arc<PgPool>>) -> impl Responder {
    let pg_pool: &PgPool = &**pool;

    match services::subs::get_all_subscription_tiers(pg_pool).await {
        Ok(tiers) => HttpResponse::Ok().json(tiers),
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().body("Error retrieving subscription plans")
        }
    }
}

#[get("/sub/plan/{id}")]
pub async fn get_plan(path: web::Path<i32>, pool: web::Data<Arc<PgPool>>) -> impl Responder {
    let id = path.into_inner();
    let pg_pool: &PgPool = &**pool;

    match services::subs::get_subscription_tier_by_id(pg_pool, id).await {
        Ok(Some(tier)) => HttpResponse::Ok().json(tier),
        Ok(None) => {
            HttpResponse::NotFound().body(format!("Subscription plan with id {} not found", id))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().body("Database error")
        }
    }
}

#[post("/sub/subscribe")]
pub async fn subscribe(
    claims: web::ReqData<Claims>,
    subscription: web::Json<CreateSubscriptionRequest>,
    pool: web::Data<Arc<PgPool>>,
    jwt_config: web::Data<JwtConfig>,
) -> impl Responder {
    let user_id = claims.user_id;
    let tier_id = subscription.tier_id;
    let pg_pool: &PgPool = &**pool;
	println!("{},{}",tier_id,user_id);
    match services::subs::create_user_subscription(pg_pool, user_id, tier_id).await {
        Ok(subscription) => {
            match middlewares::auth::update_jwt_with_subscription(
                &user_id.to_string(),
                pg_pool,
                &jwt_config,
            )
            .await
            {
                Ok(token) => HttpResponse::Created().json(serde_json::json!({
                    "subscription": subscription,
                    "token": token
                })),
                Err(_) => HttpResponse::Created().json(subscription),
            }
        }
        Err(sqlx::Error::RowNotFound) => {
            HttpResponse::BadRequest().body("Invalid subscription tier")
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().body("Error creating subscription")
        }
    }
}

#[get("/sub/current")]
pub async fn get_current_subscription(
    claims: web::ReqData<Claims>,
    pool: web::Data<Arc<PgPool>>,
) -> impl Responder {
    let user_id = claims.user_id;
    let pg_pool: &PgPool = &**pool;

    if let Some(tier_id) = claims.tier_id {
        if let Some(status) = &claims.subscription_status {
            if status == "active" {
                match services::subs::get_subscription_tier_by_id(pg_pool, tier_id).await {
                    Ok(Some(tier)) => {
                        return HttpResponse::Ok().json(serde_json::json!({
                            "subscription_status": status,
                            "tier": tier
                        }))
                    }
                    _ => {}
                }
            }
        }
    }

    match services::subs::get_user_subscription(pg_pool, user_id).await {
        Ok(Some(subscription)) => HttpResponse::Ok().json(subscription),
        Ok(None) => HttpResponse::NotFound().body("No active subscription found"),
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().body("Database error")
        }
    }
}
