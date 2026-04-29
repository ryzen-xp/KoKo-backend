use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::env;

pub async fn connect_db() -> Pool<Postgres> {
    let db_url = env::var("DB_URL").expect("failed to fetch DB URL");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("failed to connect to DB");

    println!("🛢️ Database Connected!");

    pool
}
