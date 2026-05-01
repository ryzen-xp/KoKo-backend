use std::sync::Arc;

use axum::{Router, routing::post};

use crate::AppState;
use crate::handlers::users::create_user;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/users", post(create_user))
}


