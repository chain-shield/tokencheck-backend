//! Main entry point for the CHAIN SHIELD TokenCheck Backend.
//! This module initializes the environment, logger, and sets up an HTTP server
//! that provides REST API endpoints for item management and request logging.

// HTTP server modules
mod server;

use crate::server::config::Config;
use crate::server::middlewares::auth::AuthMiddleware;
use crate::server::middlewares::logger;

use actix_cors::Cors;
use actix_web::{App, HttpServer, http::header, web};
use sqlx::PgPool;
use sqlx::migrate::MigrateDatabase;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        server::routes::auth::register,
        server::routes::auth::login,
        server::routes::auth::auth_provider,
        server::routes::auth::auth_provider_callback,
        server::routes::user::me,
        server::routes::log::report,
        server::routes::sub::get_all_plans,
        server::routes::sub::get_plan,
        server::routes::sub::subscribe,
        server::routes::sub::get_my_plan,
    ),
    info(
        title = "Web Server API",
        version = "1.0.0",
        description = "API documentation for Web Server"
    )
)]
struct ApiDoc;

use chainshield_backend::{
    utils::logging::setup_logger,
};
use dotenv::dotenv;


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

    if config_clone.console_logging_enabled {
        env_logger::init();
    }

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

        let mut app = App::new()
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
                            .service(server::routes::auth::register)
                            .service(server::routes::auth::login)
                    )
                    .service(
                        web::scope("/secured")
                            .wrap(AuthMiddleware::new(config_clone.jwt_config.clone()))
                            .service(server::routes::user::me)
                            .service(server::routes::log::report)
                            .service(server::routes::sub::get_all_plans)
                            .service(server::routes::sub::get_plan)
                            .service(server::routes::sub::subscribe)
                            .service(server::routes::sub::get_my_plan),
                    ),
            );

        if config_clone.github_client.is_configured() {
            app = app.service(
                web::scope("/api/auth")
                    .service(server::routes::auth::auth_provider)
                    .service(server::routes::auth::auth_provider_callback),
            );
        } else {
            log::warn!("GitHub OAuth is not configured. GitHub authentication routes will not be available.");
        }

        app
    })
    .bind((config.server_host.as_str(), config.server_port))?
    .workers(config.num_workers)
    .run()
    .await
}



