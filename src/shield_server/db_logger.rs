use actix_web::web;
use actix_web::{
    Error,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use chrono::Utc;
use futures::future::{LocalBoxFuture, Ready, ready};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;

/// Middleware for logging HTTP requests to a PostgreSQL database.
///
/// This middleware captures information about each request and response,
/// including method, path, status code, and query parameters, and stores
/// them in a database table named `request_logs`.
pub struct LoggerMiddleware;

impl<S, B> Transform<S, ServiceRequest> for LoggerMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = LoggerMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(LoggerMiddlewareService { service }))
    }
}

/// Service implementation for the logger middleware.
///
/// This struct holds the inner service and implements the actual request
/// processing and logging logic.
pub struct LoggerMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for LoggerMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Extract the database pool from the application data
        let pool = req
            .app_data::<web::Data<Arc<PgPool>>>()
            .expect("Database pool not found in app data")
            .clone();
        let method = req.method().to_string();
        let path = req.path().to_string();
        let params = req.query_string().to_string();

        let fut = self.service.call(req);

        Box::pin(async move {
            let pg_pool: &PgPool = &**pool;
            let res = fut.await?;
            let status = res.status().as_u16() as i32;
            let timestamp = Utc::now();

            // Log the request details to the database
            if let Err(err) = sqlx::query(
                "INSERT INTO request_logs (timestamp, method, path, status_code, params, payload) 
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(timestamp)
            .bind(&method)
            .bind(&path)
            .bind(status)
            .bind(json!({ "query": params }))
            .bind(json!({}))
            .execute(pg_pool)
            .await
            {
                eprintln!("Failed to log request: {}", err);
            }

            Ok(res)
        })
    }
}
