//! Main entry point for the CHAIN SHIELD TokenCheck Backend.
//! This module initializes the environment, logger, and performs token checking for tokens in the mainnet whitelist.
//! It demonstrates a test run where token data is fetched, token checklists are generated, and token scores are calculated using both rule-based and AI-based methods.

use anyhow::{anyhow, Result};
use chainshield_backend::{
    app_config::AI_MODEL,
    data::{
        provider_manager::get_chain_provider, token_checklist_cache::get_token_checklist,
        token_data::get_core_token_data_by_address,
    },
    token_check::{
        token_checklist::generate_token_checklist,
        token_score::{get_token_score_with_ai, get_token_score_with_rules_based_approch},
    },
	shield_server::{
		database,
		db_logger,
		db_models,
		db_models::Item
	},
    utils::logging::setup_logger,
};
use dotenv::dotenv;
use ethers::types::Address;
use log::info;

//Http_server modules
use crate::db_models::ReportFilter;
use actix_web::{get, post, put, delete, web, App, HttpServer, Responder, HttpResponse};
use sqlx::PgPool;
use std::sync::Arc;
use serde_json::json;
use chrono::Utc;
//Http_server modules

//Http server REST API functions
#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello!")
}

#[get("/item/{id}")]
async fn get_item(path: web::Path<i32>) -> impl Responder {
    let id = path.into_inner();
    let item = Item { id, name: "Sample Item".to_string() };
    HttpResponse::Ok().json(item)
}

#[post("/item")]
async fn create_item(
    item: web::Json<Item>,
    pool: web::Data<Arc<sqlx::PgPool>>,
) -> impl Responder {
    let payload = json!(item.0);
    log_request(&pool, "POST", "/item", 201, None, Some(payload)).await;
    HttpResponse::Created().json(item.0)
}

#[put("/item/{id}")]
async fn update_item(
    path: web::Path<i32>,
    item: web::Json<Item>,
    pool: web::Data<Arc<sqlx::PgPool>>,
) -> impl Responder {
    let id = path.into_inner();
    let payload = json!(item.0);
    log_request(&pool, "PUT", &format!("/item/{}", id), 200, None, Some(payload)).await;
    HttpResponse::Ok().json(item.0)
}

#[delete("/item/{id}")]
async fn delete_item(
    path: web::Path<i32>,
    pool: web::Data<Arc<sqlx::PgPool>>,
) -> impl Responder {
    let id = path.into_inner();
    log_request(&pool, "DELETE", &format!("/item/{}", id), 204, None, None).await;
    HttpResponse::NoContent().finish()
}

#[get("/report")]
async fn report(
    pool: web::Data<Arc<sqlx::PgPool>>,
    filter: web::Query<ReportFilter>,
) -> impl Responder {
    let mut query_conditions = Vec::new();
    let mut params = Vec::new();
    let mut param_count = 1;
    
    let mut query_base = "SELECT * FROM request_logs".to_string();
    
    if let Some(method) = &filter.method {
        query_conditions.push(format!("method = ${}", param_count));
        params.push(method.clone());
        param_count += 1;
    }
    if let Some(status_code) = filter.status_code {
        query_conditions.push(format!("status_code = ${}::INTEGER", param_count));
        params.push(status_code.to_string());
        param_count += 1;
    }
    if let Some(path) = &filter.path {
        query_conditions.push(format!("path = ${}", param_count));
        params.push(path.clone());
    }

    if !query_conditions.is_empty() {
        query_base.push_str(" WHERE ");
        query_base.push_str(&query_conditions.join(" AND "));
    }

    let mut query = sqlx::query_as::<_, db_models::LogEntry>(&query_base);
    
    for param in params {
        query = query.bind(param);
    }
    
    let pg_pool: &PgPool = &**pool;
    let logs = query
        .fetch_all(pg_pool)
        .await
        .expect("Failed to fetch logs");

    HttpResponse::Ok().json(logs)
}

async fn log_request(
    pool: &Arc<sqlx::PgPool>,
    method: &str,
    path: &str,
    status_code: i32,
    params: Option<serde_json::Value>,
    payload: Option<serde_json::Value>,
) {
    let timestamp = Utc::now().naive_utc();
    if let Err(err) = sqlx::query(
        "INSERT INTO request_logs (timestamp, method, path, status_code, params, payload) 
         VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(timestamp)
    .bind(method)
    .bind(path)
    .bind(status_code)
    .bind(params.unwrap_or(json!({})))
    .bind(payload.unwrap_or(json!({})))
    .execute(&**pool)
    .await {
        eprintln!("Failed to log request: {}", err);
    }
}


/// Main function that iterates over tokens, performs token checks, and prints token scores using both rule-based and AI-based approaches.
///
/// # Errors
/// Returns an error if any step in token processing or blockchain communication fails.
//#[tokio::main]
//async fn main() -> Result<()> {
//    dotenv().ok();
//    setup_logger().expect("Failed to initialize logger.");

//    Ok(())
//}

//#[tokio::main]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
	dotenv().ok();
    setup_logger().expect("Failed to initialize logger.");

    let max_connections = 10;
    let pool = database::establish_connection(max_connections).await;
    let pool = Arc::new(pool);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(db_logger::LoggerMiddleware)
            .service(index)
            .service(create_item)
            .service(get_item)
            .service(update_item)
            .service(delete_item)
            .service(report)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}