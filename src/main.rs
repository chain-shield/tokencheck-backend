mod server;

use crate::server::config::Config;
use crate::server::middlewares::auth::AuthMiddleware;
use crate::server::middlewares::logger;

use actix_cors::Cors;
use actix_session::SessionMiddleware;
use actix_session::config::PersistentSession;
use actix_session::storage::CookieSessionStore;
use actix_web::cookie::time::Duration;
use actix_web::cookie::{Key, SameSite};
use actix_web::{App, HttpServer, http::header, web};
use colored::Colorize;
use sqlx::PgPool;
use sqlx::migrate::MigrateDatabase;
use std::fs::File;
use std::sync::Arc;
// use utoipa::OpenApi;
// use utoipa_swagger_ui::SwaggerUi;

pub fn setup_logger() -> Result<(), fern::InitError> {
    File::create("snipper.log").map_err(fern::InitError::Io)?;

    fern::Dispatch::new()
        .format(|out, message, record| {
            let color = match record.level() {
                log::Level::Info => "green",
                log::Level::Warn => "yellow",
                log::Level::Error => "red",
                log::Level::Debug => "magenta",
                log::Level::Trace => "bright black",
            };
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%H:%M:%S]"),
                record.target(),
                record.level().to_string().color(color),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .level_for("ethers_providers", log::LevelFilter::Off)
        .level_for("hyper", log::LevelFilter::Off)
        .chain(std::io::stdout())
        .chain(fern::log_file("snipper.log")?)
        .apply()?;
    Ok(())
}

// #[derive(OpenApi)]
// #[openapi(
//     paths(
//         server::routes::log::report,
//         server::routes::auth::register,
//         server::routes::auth::login,
//         server::routes::auth::auth_provider,
//         server::routes::auth::auth_provider_callback,
//         server::routes::user::me,
//         server::routes::session::get_session,
//         server::routes::sub::get_all_plans,
//         server::routes::sub::get_plan,
//         server::routes::sub::subscribe,
//         server::routes::sub::get_my_plan,
//         server::routes::pay::one_time_checkout,
//         server::routes::pay::subscribe_checkout,
//         server::routes::pay::subscribe_custom_checkout,
//         server::routes::pay::webhook
//     ),
//     info(
//         title = "Web Server API",
//         version = "1.0.0",
//         description = "API documentation for Web Server"
//     )
// )]
// struct ApiDoc;

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

    if config_clone.console_logging_enabled {
        setup_logger().expect("Failed to set up logger");
    }

    let pool = setup_database(&config.database_url)
        .await
        .expect("Failed to set up database");
    let pool = Arc::new(pool);

    HttpServer::new(move || {
        let secret_key = Key::generate();
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

        let app = App::new()
            .wrap(logger::LoggerMiddleware::new(
                config_clone.console_logging_enabled,
            ))
            .wrap(cors)
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key)
                    .cookie_name("auth_session".to_string())
                    .cookie_secure(false) // change to true in production when HTTPS is enabled
                    .cookie_same_site(SameSite::Lax)
                    .cookie_domain(None) // Let browser determine domain
                    .session_lifecycle(
                        PersistentSession::default().session_ttl(Duration::hours(24)),
                    )
                    .build(),
            )
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config_clone.clone()))
            // .service(
            //     #[cfg(debug_assertions)]
            //     SwaggerUi::new("/swagger-ui/{_:.*}")
            //         .url("/api-docs/openapi.json", ApiDoc::openapi()),
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
                            .service(
                                web::scope("/sub")
                                    .service(server::routes::sub::get_all_plans)
                                    .service(server::routes::sub::subscribe)
                                    .service(server::routes::sub::get_my_subscription)
                                    .service(server::routes::sub::update_auto_renew),
                            )
                            .service(
                                web::scope("/pay")
                                    .service(server::routes::pay::refund)
                                    .service(server::routes::pay::get_subscription_payment)
                                    .service(server::routes::pay::get_payment_intents)
                            ),
                    )
                    .service(web::scope("/pay").service(server::routes::pay::webhook)),
            );

        app
    })
    .bind((config.server_host.as_str(), config.server_port))?
    .workers(config.num_workers)
    .run()
    .await
}
