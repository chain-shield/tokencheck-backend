use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use actix_web::web;
use chrono::Utc;
use futures::future::{ready, LocalBoxFuture, Ready};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;

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
        let pool = req.app_data::<web::Data<Arc<PgPool>>>().unwrap().clone();
        let method = req.method().to_string();
        let path = req.path().to_string();
        let params = req.query_string().to_string();

        let fut = self.service.call(req);

        Box::pin(async move {
            let pg_pool: &PgPool = &**pool;
            let res = fut.await?;
            let status = res.status().as_u16() as i32;
            let timestamp = Utc::now();

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
