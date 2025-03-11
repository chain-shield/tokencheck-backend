//! Main entry point for the CHAIN SHIELD TokenCheck Backend.
//! This module initializes the environment, logger, and sets up an HTTP server
//! that provides REST API endpoints for item management and request logging.

// HTTP server modules
mod middlewares;
mod models;
mod routes;
mod services;

use crate::middlewares::auth::{AuthMiddleware, JwtConfig};
use crate::middlewares::logger;

use actix_cors::Cors;
use actix_web::{get, http::header, web, App, HttpResponse, HttpServer, Responder};
use sqlx::migrate::MigrateDatabase;
use sqlx::PgPool;
use std::env;
use std::sync::Arc;

use chainshield_backend::{
    utils::logging::setup_logger,
};
use dotenv::dotenv;


#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello!")
}

async fn setup_database(database_url: &str) -> Result<PgPool, Box<dyn std::error::Error>> {
    if !sqlx::Postgres::database_exists(database_url).await? {
        sqlx::Postgres::create_database(database_url).await?;
    }
    let pool = PgPool::connect(database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	// Load environment variables from .env file
    dotenv().ok();
    // Initialize the logger
    setup_logger().expect("Failed to initialize logger.");

    // Load data from .env
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let ip: String = env::var("IP").expect("IP must be set");
    let port: u16 = env::var("PORT")
        .expect("PORT must be set")
        .parse()
        .expect("Failed to parse PORT as u16");
    let num_workers: usize = env::var("WORKERS")
        .expect("WORKERS must be set")
        .parse()
        .expect("Failed to parse WORKERS as usize");

    // Get CORS allowed origins from environment or use default
    let cors_origin =
        env::var("CORS_ALLOWED_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".to_string());

    // Connect psql databse
    let pool = setup_database(&database_url)
        .await
        .expect("Failed to set up database");
    let pool = Arc::new(pool);

    // Create JWT config
    let jwt_config = JwtConfig::from_env();

    let server = HttpServer::new(move || {
        let auth_middleware = AuthMiddleware::new(jwt_config.clone());

        // Configure CORS
        let cors = Cors::default()
            .allowed_origin(&cors_origin)
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                header::AUTHORIZATION,
                header::CONTENT_TYPE,
                header::ACCEPT,
            ])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(jwt_config.clone()))
            .wrap(cors)
            .wrap(logger::LoggerMiddleware)
            .service(
                web::scope("/api")
                    .service(index)
                    .service(
                        web::scope("/auth")
                            .service(routes::auth::register)
                            .service(routes::auth::login),
                    )
                    .service(
                        web::scope("/secured")
                            .wrap(auth_middleware)
                            .service(routes::user::me)
                            .service(routes::item::get_all_items)
                            .service(routes::item::create_item)
                            .service(routes::item::get_item)
                            .service(routes::item::update_item)
                            .service(routes::item::delete_item)
                            .service(routes::report::report)
                            .service(routes::subs::get_all_plans)
                            .service(routes::subs::get_plan)
                            .service(routes::subs::subscribe)
                            .service(routes::subs::get_current_subscription),
                    ),
            )
    })
    .bind((ip.as_str(), port))?
    .workers(num_workers)
    .run();

    println!("Server running at http://{}:{}", ip, port);

    server.await
}