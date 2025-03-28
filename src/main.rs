//! Main entry point for the CHAIN SHIELD TokenCheck Backend.
//! This module initializes the environment, logger, and sets up an HTTP server
//! that provides REST API endpoints for item management and request logging.

// HTTP server modules
use actix_cors::Cors;
use actix_session::config::PersistentSession;
use actix_session::storage::CookieSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::time::Duration;
use actix_web::cookie::{Key, SameSite};
use actix_web::http::header;
use actix_web::{dev::Service, web, App, HttpServer};
use log::{debug, info};
use sqlx::migrate::MigrateDatabase;
use sqlx::PgPool;
use std::sync::Arc;
use tokencheck_backend::env_config::Config;
use tokencheck_backend::server;
use tokencheck_backend::server::middlewares::auth::AuthMiddleware;
use utoipa::OpenApi;
// use utoipa_swagger_ui::SwaggerUi;

use dotenv::dotenv;
use tokencheck_backend::utils::logging::setup_logger;

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
        server::routes::session::get_session
    ),
    info(
        title = "Web Server API",
        version = "1.0.0",
        description = "API documentation for Web Server"
    )
)]
struct _ApiDoc;

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
    let config = Arc::new(Config::from_env());
    let config_clone = config.clone();

    dotenv().ok();

    if config_clone.console_logging_enabled {
        setup_logger().expect("Failed to set up logger");
    }

    let pool = setup_database(&config.database_url)
        .await
        .expect("Failed to set up database");
    let pool = Arc::new(pool);

    // Clone the string before splitting it to keep the string alive
    let origin = config.cors_allowed_origin.clone();
    info!("cors origins {}", origin);

    // if contains localhost (running locally) then its does not need secure cookies
    let cookie_secure = !origin.contains("localhost");
    info!("cookie_secure => {}", cookie_secure);

    HttpServer::new(move || {
        let secret_key = Key::derive_from(config_clone.jwt_config.secret.as_bytes());
        let cors = Cors::default()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                header::AUTHORIZATION,
                header::CONTENT_TYPE,
                header::ACCEPT,
                header::COOKIE,
                header::SET_COOKIE,
            ])
            .allowed_origin(&origin)
            .expose_headers(&[header::SET_COOKIE])
            .supports_credentials()
            .max_age(3600);

        let app = App::new()
            .wrap(cors)
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key)
                    .cookie_name("auth_session".to_string())
                    .cookie_secure(cookie_secure) // change to true in production when HTTPS is enabled
                    .cookie_same_site(if cookie_secure {
                        SameSite::None
                    } else {
                        SameSite::Lax
                    }) // Use Lax for better browser compatibility
                    .cookie_http_only(true)
                    .cookie_domain(if config_clone.environment == "production" {
                        Some(".tokencheck.ai".to_string())
                    // gcp domain , switch to
                    // tokencheck.ai  once live
                    } else {
                        None // let browser set
                    })
                    .session_lifecycle(
                        PersistentSession::default().session_ttl(Duration::hours(24)),
                    )
                    .build(),
            )
            .wrap_fn(|req, srv| {
                // --- LOG INCOMING REQUEST HEADERS ---
                debug!("--- Incoming Request Headers ---");
                for (name, value) in req.headers().iter() {
                    println!("{}: {:?}", name, value);
                }

                let fut = srv.call(req);
                async move {
                    let res = fut.await?;

                    // --- LOG OUTGOING RESPONSE HEADERS ---
                    debug!("--- Outgoing Response Headers ---");
                    for (name, value) in res.headers().iter() {
                        debug!("{}: {:?}", name, value);
                    }

                    Ok(res)
                }
            })
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config_clone.clone()))
            // .service(
            //     #[cfg(debug_assertions)]
            //     SwaggerUi::new("/swagger-ui/{_:.*}")
            //         .url("/api-docs/openapi.json", ApiDoc::openapi()),
            //     #[cfg(not(debug_assertions))]
            //     {
            //         web::scope("/swagger-ui") // Empty scope as a placeholder
            //     },
            // )
            .service(
                web::scope("/api")
                    .service(server::routes::session::get_session)
                    .service(
                        web::scope("/auth")
                            .service(server::routes::auth::register)
                            .service(server::routes::auth::login)
                            .service(server::routes::auth::auth_provider)
                            .service(server::routes::auth::auth_provider_callback),
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

        app
    })
    .bind((config.server_host.as_str(), config.server_port))?
    .workers(config.num_workers)
    .run()
    .await
}
