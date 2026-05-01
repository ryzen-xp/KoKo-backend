use axum::Router;
use database::connect_db;
use dotenv::dotenv;
use errors::AppError;
use routers::health::routes as health_routes;
use routers::users::routes as user_routes;
use std::{env, sync::Arc};

#[derive(Clone)]
struct AppState {
    db: sqlx::Pool<sqlx::Postgres>,
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenv().ok();

    let port = env::var("PORT")
        .map_err(|e| AppError::Internal(anyhow::anyhow!("[ENV]: failed to load PORT: {}", e)))?;
    let host = env::var("HOST")
        .map_err(|e| AppError::Internal(anyhow::anyhow!("[ENV]: failed to load HOST: {}", e)))?;

    let db_pool = connect_db().await?;

    let shared_state = Arc::new(AppState { db: db_pool });

    let api_routes = user_routes().merge(health_routes());
    let app = Router::new().nest("/api", api_routes).with_state(shared_state);


    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    println!("🌎 Server running at {}", addr);

    axum::serve(listener, app)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(())
}

mod database;
mod errors;
mod handlers;
mod routers;
mod models;
