use actix_web::{HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    server::{
        dtos::key::{ApiKeyDto, CreateApiKeyResponse},
        misc::error::Res,
        models::api::ApiKey,
    },
    utils::encryption::EncryptionService,
};

pub async fn delete_user_api_key_from_db(
    id: &Uuid,
    pool: &PgPool,
    user_id: &Uuid,
) -> Res<impl Responder> {
    // Verify that the key exists and belongs to the user
    let key = sqlx::query!(
        r#"
            SELECT user_id
            FROM api_keys
            WHERE id = $1
            "#,
        id
    )
    .fetch_optional(pool)
    .await?;

    match key {
        Some(key) => {
            if key.user_id != *user_id {
                return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Unauthorized",
                    "message": "You are Not Authorized to Delete the Key"
                })));
            }
        }
        None => {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "400 Not Found",
                "message": "Cound Not find Api Key"
            })))
        }
    }

    sqlx::query!(
        r#"
        DELETE FROM api_keys
        WHERE id = $1 
        "#,
        id
    )
    .execute(pool)
    .await?;

    Ok(HttpResponse::Ok().finish())
}

pub async fn get_user_api_keys_from_db(pool: &PgPool, user_id: &Uuid) -> Res<impl Responder> {
    let encryption = EncryptionService::new()?;
    let keys = sqlx::query!(
        r#"
            SELECT id, key_encrypted, name, status, created_at, last_used, permissions
            FROM api_keys
            WHERE user_id = $1
            "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    let mut api_keys = Vec::new();
    for key in keys {
        let plaintext_key = encryption.decrypt(&key.key_encrypted)?;

        api_keys.push(ApiKeyDto {
            id: key.id,
            key: plaintext_key,
            name: key.name,
            status: key.status,
            created_at: key.created_at,
        });
    }

    Ok(HttpResponse::Created().json(api_keys))
}

pub async fn create_api_key_and_save_to_db(
    api_key_name: String,
    pool: &PgPool,
    user_id: &Uuid,
) -> Res<impl Responder> {
    let encryption = EncryptionService::new()?;

    // Validate input
    if api_key_name.is_empty() || api_key_name.len() > 100 {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid request",
            "message": "Name is required and must be <= 100 characters"
        })));
    }

    // Generate a secure API key
    let key = format!("eth_{}", hex::encode(rand::random::<[u8; 32]>()));
    let key_encrypted = match encryption.encrypt(&key) {
        Ok(encrypted) => encrypted,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal Server Error",
                "message": e.to_string()
            })))
        }
    };

    // TODO - UPDATE this is placeholder value
    let permissions = serde_json::json!({
        "endpoints": ["/token/assessment"],
        "rate_limit": 100
    });

    let new_key = sqlx::query_as!(
    ApiKey,
    r#"
        INSERT INTO api_keys (user_id, key_encrypted, name, permissions)
        VALUES ($1, $2, $3, $4)
        RETURNING id, user_id, key_encrypted as "encrypted_key!", name, status, created_at, last_used, permissions
        "#,
    user_id,
    key_encrypted,
    api_key_name,
    permissions
    )
    .fetch_one(pool)
    .await?;

    Ok(HttpResponse::Created().json(CreateApiKeyResponse {
        id: new_key.id,
        user_id: new_key.id,
        name: new_key.name,
        key,
        status: new_key.status,
        created_at: new_key.created_at,
        permissions: new_key.permissions,
    }))
}
