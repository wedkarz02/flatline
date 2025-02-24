use std::sync::Arc;

use axum::Router;
use tower_http::trace::TraceLayer;

use crate::AppState;

pub mod user;

pub fn create_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .nest("/api/v1/users", user::create_routes(Arc::clone(&state)))
        .layer(TraceLayer::new_for_http())
}
