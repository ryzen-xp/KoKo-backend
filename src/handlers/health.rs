use std::sync::Arc;

use axum::{

    extract::State,
    http::StatusCode,
};

use crate::{AppError, AppState, handlers::health};




pub async  fn get_health_status(State(state): State<Arc<AppState>>)->(StatusCode , String) {
     
     match  sqlx::query("SELECT 1").execute(&state.db).await {
         Ok(_) => (StatusCode::OK , String::from("Database is Healthy !!")), 
         Err(_) => (StatusCode::SERVICE_UNAVAILABLE , String::from("Database  blast, check it ASAP !!!"))
     }
}
