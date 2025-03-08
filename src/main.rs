//! Main entry point for the CHAIN SHIELD TokenCheck Backend.
//! This module initializes the environment, logger, and sets up an HTTP server
//! that provides REST API endpoints for item management and request logging.

use chainshield_backend::{
    shield_server::{database, db_logger, db_models, db_models::Item},
    utils::logging::setup_logger,
};
use dotenv::dotenv;

// HTTP server modules
use crate::db_models::ReportFilter;
use actix_web::{App, HttpResponse, HttpServer, Responder, delete, get, post, put, web};
use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;

// REST API endpoint handlers

/// Root endpoint that returns a simple greeting
#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello!")
}

/// Retrieves an item by its ID
#[get("/item/{id}")]
async fn get_item(path: web::Path<i32>) -> impl Responder {
    let id = path.into_inner();
    let item = Item {
        id,
        name: "Sample Item".to_string(),
    };
    HttpResponse::Ok().json(item)
}

/// Creates a new item and logs the request
#[post("/item")]
async fn create_item(item: web::Json<Item>, pool: web::Data<Arc<sqlx::PgPool>>) -> impl Responder {
    let payload = json!(item.0);
    log_request(&pool, "POST", "/item", 201, None, Some(payload)).await;
    HttpResponse::Created().json(item.0)
}

/// Updates an existing item by ID and logs the request
#[put("/item/{id}")]
async fn update_item(
    path: web::Path<i32>,
    item: web::Json<Item>,
    pool: web::Data<Arc<sqlx::PgPool>>,
) -> impl Responder {
    let id = path.into_inner();
    let payload = json!(item.0);
    log_request(
        &pool,
        "PUT",
        &format!("/item/{}", id),
        200,
        None,
        Some(payload),
    )
    .await;
    HttpResponse::Ok().json(item.0)
}

/// Deletes an item by ID and logs the request
#[delete("/item/{id}")]
async fn delete_item(path: web::Path<i32>, pool: web::Data<Arc<sqlx::PgPool>>) -> impl Responder {
    let id = path.into_inner();
    log_request(&pool, "DELETE", &format!("/item/{}", id), 204, None, None).await;
    HttpResponse::NoContent().finish()
}

/// Generates a report of request logs with optional filtering
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
        .expect("Failed to fetch logs"); // Consider replacing with ? operator

    HttpResponse::Ok().json(logs)
}

/// Logs HTTP request details to the database
///
/// # Parameters
/// * `pool` - Database connection pool
/// * `method` - HTTP method (GET, POST, etc.)
/// * `path` - Request path
/// * `status_code` - HTTP status code
/// * `params` - Optional query parameters
/// * `payload` - Optional request body
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
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(timestamp)
    .bind(method)
    .bind(path)
    .bind(status_code)
    .bind(params.unwrap_or(json!({})))
    .bind(payload.unwrap_or(json!({})))
    .execute(&**pool)
    .await
    {
        eprintln!("Failed to log request: {}", err);
    }
}

/// Main function that initializes the environment and starts the HTTP server.
///
/// # Errors
/// Returns an error if the server fails to start or bind to the specified address.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    // Initialize the logger
    setup_logger().expect("Failed to initialize logger.");

    // Establish database connection with connection pool
    let max_connections = 10;
    let pool = database::establish_connection(max_connections).await;
    let pool = Arc::new(pool);

    // Configure and start the HTTP server
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
