//! Main entry point for the CHAIN SHIELD TokenCheck Backend.
//! This module initializes the environment, logger, and sets up an HTTP server
//! that provides REST API endpoints for item management and request logging.

// HTTP server modules
mod config;
mod middlewares;
mod models;
mod repo;
mod routes;
mod services;
mod misc;
mod dtos;

use crate::config::Config;
use crate::middlewares::auth::AuthMiddleware;
use crate::middlewares::logger;

use actix_cors::Cors;
use actix_web::{http::header, web, App, HttpServer};
use sqlx::migrate::MigrateDatabase;
use sqlx::PgPool;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use chainshield_backend::{
    utils::logging::setup_logger,
};
use dotenv::dotenv;

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::auth::register,
        routes::auth::login,
        routes::auth::auth_provider,
        routes::auth::auth_provider_callback,
        routes::user::me,
        routes::log::report,
        routes::sub::get_all_plans,
        routes::sub::get_plan,
        routes::sub::subscribe,
        routes::sub::get_my_plan,
    ),
    info(
        title = "Web Server API",
        version = "1.0.0",
        description = "API documentation for Web Server"
    )
)]
struct ApiDoc;

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
    dotenv().ok();
    // Initialize the logger
    setup_logger().expect("Failed to initialize logger.");
	// Load environment variables from .env file
    let config = Arc::new(Config::from_env());
    let config_clone = config.clone();

    env_logger::init();

    let pool = setup_database(&config.database_url)
        .await
        .expect("Failed to set up database");
    let pool = Arc::new(pool);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&config_clone.cors_allowed_origin)
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                header::AUTHORIZATION,
                header::CONTENT_TYPE,
                header::ACCEPT,
            ])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .wrap(logger::LoggerMiddleware::new(
                config_clone.console_logging_enabled,
            ))
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config_clone.clone()))
            .service(
                #[cfg(debug_assertions)]
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi()),
            )
            .service(
                web::scope("/api")
                    .service(
                        web::scope("/auth")
                            .service(routes::auth::register)
                            .service(routes::auth::login)
                            .service(routes::auth::auth_provider)
                            .service(routes::auth::auth_provider_callback),
                    )
                    .service(
                        web::scope("/secured")
                            .wrap(AuthMiddleware::new(config_clone.jwt_config.clone()))
                            .service(routes::user::me)
                            .service(routes::log::report)
                            .service(routes::sub::get_all_plans)
                            .service(routes::sub::get_plan)
                            .service(routes::sub::subscribe)
                            .service(routes::sub::get_my_plan),
                    ),
            )
    })
    .bind((config.server_host.as_str(), config.server_port))?
    .workers(config.num_workers)
    .run()
    .await
}
