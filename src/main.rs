use axum::Router;
use database::connect_db;
use dotenv::dotenv;
use std::{env, sync::Arc};

#[derive(Clone)]
struct AppState {
    db: sqlx::Pool<sqlx::Postgres>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let port = env::var("PORT").expect("failed to load PORT");
    let host = env::var("HOST").expect("failed to load HOST");

    let db_pool = connect_db().await;

    let shared_state = Arc::new(AppState { db: db_pool });

    let app = Router::new().with_state(shared_state);

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to start server");

    println!("🌎 Server running at {}", addr);

    axum::serve(listener, app).await.unwrap();
}

mod database;
