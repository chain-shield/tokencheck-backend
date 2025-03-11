use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    models::CreateItemRequest,
    services,
};

#[get("/items")]
pub async fn get_all_items(pool: web::Data<Arc<PgPool>>) -> impl Responder {
    let pg_pool: &PgPool = &**pool;

    match services::item::get_all_items(pg_pool).await {
        Ok(items) => HttpResponse::Ok().json(items),
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().body("Error retrieving items")
        }
    }
}

#[get("/item/{id}")]
pub async fn get_item(path: web::Path<Uuid>, pool: web::Data<Arc<PgPool>>) -> impl Responder {
    let id = path.into_inner();
    let pg_pool: &PgPool = &**pool;

    match services::item::get_item_by_id(pg_pool, id).await {
        Ok(Some(item)) => HttpResponse::Ok().json(item),
        Ok(None) => HttpResponse::NotFound().body(format!("Item with id {} not found", id)),
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().body("Database error")
        }
    }
}

#[post("/item")]
pub async fn create_item(
    item: web::Json<CreateItemRequest>,
    pool: web::Data<Arc<PgPool>>,
) -> impl Responder {
    let pg_pool: &PgPool = &**pool;

    match services::item::create_item(pg_pool, item.name.clone()).await {
        Ok(created_item) => HttpResponse::Created().json(created_item),
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().body("Error creating item")
        }
    }
}

#[put("/item/{id}")]
pub async fn update_item(
    path: web::Path<Uuid>,
    item: web::Json<CreateItemRequest>,
    pool: web::Data<Arc<PgPool>>,
) -> impl Responder {
    let id = path.into_inner();
    let pg_pool: &PgPool = &**pool;

    match services::item::update_item_by_id(pg_pool, id, item.name.clone()).await {
        Ok(Some(updated_item)) => HttpResponse::Ok().json(updated_item),
        Ok(None) => HttpResponse::NotFound().body(format!("Item with id {} not found", id)),
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().body("Error updating item")
        }
    }
}

#[delete("/item/{id}")]
pub async fn delete_item(path: web::Path<Uuid>, pool: web::Data<Arc<PgPool>>) -> impl Responder {
    let id = path.into_inner();
    let pg_pool: &PgPool = &**pool;

    match services::item::delete_item_by_id(pg_pool, id).await {
        Ok(Some(())) => HttpResponse::NoContent().finish(),
        Ok(None) => HttpResponse::NotFound().body(format!("Item with id {} not found", id)),
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().body("Error deleting item")
        }
    }
}
