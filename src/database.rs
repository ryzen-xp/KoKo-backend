use crate::errors::AppError;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::env;
use anyhow::anyhow;

pub async fn connect_db() -> Result<Pool<Postgres>, AppError> {
    let db_url = env::var("DB_URL")
        .map_err(|e| AppError::Internal(anyhow!("[ENV]: failed to fetch DB URL: {}", e)))?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    println!("🛢️ Database Connected!");

    Ok(pool)
}
