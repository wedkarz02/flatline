use std::sync::Arc;

use axum::{extract::State, routing::get, Extension, Router};

use crate::{
    error::ApiError,
    routes::{extractors::ApiVersion, ApiResponse},
    services::{self, auth::AuthError, jwt::Claims},
    ApiState,
};

async fn delete_expired_jwt(
    State(state): State<Arc<ApiState>>,
    version: ApiVersion,
    Extension(claims): Extension<Claims>,
) -> Result<ApiResponse, ApiError> {
    if !claims.admin {
        return Err(AuthError::Forbidden.into());
    }

    let deleted_count = state.db.refresh_tokens().delete_expired().await?;

    ApiResponse::builder()
        .with_api_version(version)
        .with_message("deleted expired refresh tokens")
        .with_payload(serde_json::json!({"deleted_count": deleted_count}))
        .build()
        .as_ok()
}

pub fn create_routes(state: Arc<ApiState>) -> Router {
    let protected_routes = Router::new()
        .route("/delete-expired-jwt", get(delete_expired_jwt))
        .layer(axum::middleware::from_fn(services::auth::auth_guard))
        .layer(Extension(state.clone()));

    Router::new().merge(protected_routes).with_state(state)
}
