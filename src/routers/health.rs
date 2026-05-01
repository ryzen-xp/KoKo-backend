use std::sync::Arc;

use axum::{Router, routing::get};

use crate::AppState;
use crate::handlers::health::get_health_status;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/health", get(get_health_status))
}