use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::env;

pub async fn establish_connection(max_connections: u32) -> PgPool {
    dotenvy::dotenv().ok();
	let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS request_logs (
            id SERIAL PRIMARY KEY,
            timestamp TIMESTAMP NOT NULL,
            method VARCHAR(10) NOT NULL,
            path TEXT NOT NULL,
            status_code INT NOT NULL,
            params JSONB,
            payload JSONB
        )"
    )
    .execute(&pool)
    .await
    .expect("Failed to create request_logs table");

    pool
}
