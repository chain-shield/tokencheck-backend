use sqlx::PgPool;

use crate::models::Item;

type Result<T> = std::result::Result<T, sqlx::Error>;

pub async fn get_all_items(pool: &PgPool) -> Result<Vec<Item>> {
    sqlx::query_as!(Item, "SELECT id, name FROM items")
        .fetch_all(pool)
        .await
}

pub async fn get_item_by_id(pool: &PgPool, id: uuid::Uuid) -> Result<Option<Item>> {
    sqlx::query_as!(Item, "SELECT id, name FROM items WHERE id = $1", id)
        .fetch_optional(pool)
        .await
}

pub async fn create_item(pool: &PgPool, name: String) -> Result<Item> {
    sqlx::query_as!(
        Item,
        "INSERT INTO items (name) VALUES ($1) RETURNING id, name",
        name
    )
    .fetch_one(pool)
    .await
}

pub async fn update_item_by_id(pool: &PgPool, id: uuid::Uuid, name: String) -> Result<Option<Item>> {
    sqlx::query_as!(
        Item,
        "UPDATE items SET name = $1 WHERE id = $2 RETURNING id, name",
        name,
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn delete_item_by_id(pool: &PgPool, id: uuid::Uuid) -> Result<Option<()>> {
    let result = sqlx::query!("DELETE FROM items WHERE id = $1", id)
        .execute(pool)
        .await?;
    match result.rows_affected() {
        0 => Ok(None),
        _ => Ok(Some(())),
    }
}