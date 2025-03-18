use sqlx::PgPool;

use crate::{misc::error::Res, models::log::Log, repo};

pub async fn get_report(
    pool: &PgPool,
    method: Option<String>,
    code: Option<i32>,
    path: Option<String>,
) -> Res<Vec<Log>> {
    repo::log::get_report(pool, method, code, path).await
}

pub async fn insert_log(pool: &PgPool, log: Log) -> Res<()> {
    repo::log::insert_log(pool, log).await
}