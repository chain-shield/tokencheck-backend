use actix_web::{delete, get, post, web, HttpResponse, Responder};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::server::{
    dtos::key::{ApiKeyDto, CreateApiKeyRequest, CreateApiKeyResponse},
    misc::error::Res,
    models::auth::Claims,
    services::key::{
        create_api_key_and_save_to_db, delete_user_api_key_from_db, get_user_api_keys_from_db,
    },
};

#[utoipa::path(
    get,
    path = "/api/secured/key",
    tag = "Key",
    summary = "Get list of user api keys",
    description = "Get array of user api keys that are saved in db",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Api keys successfully returned", body = Vec<ApiKeyDto>),
        (status = 400, description = "Invalid request to get user API keys"),
        (status = 500, description = "Database error")
    )
)]
#[get("/key/get-all")]
pub async fn get_all_user_api_keys(
    claims: web::ReqData<Claims>,
    pool: web::Data<Arc<PgPool>>,
) -> Res<impl Responder> {
    let user_id = claims.user_id;
    let pg_pool: &PgPool = &**pool;

    let user_keys = get_user_api_keys_from_db(pg_pool, &user_id).await?;

    Ok(user_keys)
}

#[utoipa::path(
    post,
    path = "/api/secured/key",
    tag = "Key",
    summary = "POST to create new API Key",
    description = "Generates new API key for user and saves encrypted version in database",
    request_body = CreateApiKeyRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 201, description = "Api key successfully created", body = CreateApiKeyResponse),
        (status = 400, description = "Invalid request to create API key"),
        (status = 500, description = "Database error")
    )
)]
#[post("/key/generate")]
pub async fn generate_api_key(
    create_api_key_request: web::Json<CreateApiKeyRequest>,
    claims: web::ReqData<Claims>,
    pool: web::Data<Arc<PgPool>>,
) -> Res<impl Responder> {
    let user_id = claims.user_id;
    let api_key_name = create_api_key_request.name.clone();
    let pg_pool: &PgPool = &**pool;

    // let subscription = services::sub::get_user_sub(pg_pool, &user_id).await?;
    let new_key = create_api_key_and_save_to_db(api_key_name.clone(), pg_pool, &user_id).await?;

    // Return the CreateApiKeyRequest as the response
    Ok(new_key)
}

#[utoipa::path(
    delete,
    path = "/api/secured/key",
    tag = "Key",
    summary = "Delete user api key",
    description = "Delete Api key for user from database",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = String, Path, description = "API key unique identifier")  // Add param documentation
    ),
    responses(
        (status = 204, description = "API key successfully deleted"),
        (status = 400, description = "Invalid request to delete key"),
        (status = 404, description = "API key not found"),
        (status = 401, description = "Unauthorized to delete key"),
    )
)]
#[delete("/key/delete/{id}")]
pub async fn delete_user_api_keys(
    path: web::Path<String>,
    claims: web::ReqData<Claims>,
    pool: web::Data<Arc<PgPool>>,
) -> Res<impl Responder> {
    let user_id = claims.user_id;
    let pg_pool: &PgPool = &**pool;
    let id = path.into_inner();

    // Parse the string to UUID
    let id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Bad Request",
                "message": "Invalid UUID format"
            })))
        }
    };

    delete_user_api_key_from_db(&id, pg_pool, &user_id).await?;

    Ok(HttpResponse::Ok().finish())
}
