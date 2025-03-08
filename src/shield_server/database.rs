use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::env;

/// Establishes a connection to the PostgreSQL database and initializes the schema.
///
/// This function reads the database connection string from the `DATABASE_URL` environment
/// variable, creates a connection pool with the specified maximum number of connections,
/// and ensures the required database tables exist.
///
/// # Arguments
///
/// * `max_connections` - The maximum number of connections to maintain in the pool
///
/// # Returns
///
/// A `Result` containing the database connection pool if successful, or an error if the
/// connection fails or schema initialization fails.
///
/// # Errors
///
/// This function will return an error if:
/// - The `DATABASE_URL` environment variable is not set
/// - The connection to the database fails
/// - The table creation query fails
pub async fn establish_connection(max_connections: u32) -> Result<PgPool, sqlx::Error> {
    // Load environment variables from .env file if present
    dotenvy::dotenv().ok();

    // Get database URL from environment variables
    let database_url = env::var("DATABASE_URL").map_err(|_| {
        sqlx::Error::Configuration("DATABASE_URL environment variable must be set".into())
    })?;

    // Create a connection pool
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(&database_url)
        .await?;

    // Initialize database schema - create request_logs table if it doesn't exist
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS request_logs (
            id SERIAL PRIMARY KEY,
            timestamp TIMESTAMP NOT NULL,
            method VARCHAR(10) NOT NULL,
            path TEXT NOT NULL,
            status_code INT NOT NULL,
            params JSONB,
            payload JSONB
        )",
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}
