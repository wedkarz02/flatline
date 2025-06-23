use std::sync::Arc;

use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use serde::Deserialize;
use serde_json::json;

use crate::{
    error::ApiError,
    models::user::Role,
    routes::{extractors::ApiVersion, ApiResponse},
    services, ApiState,
};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct AuthPayload {
    pub username: String,
    pub password: String,
}

pub async fn register(
    version: ApiVersion,
    State(state): State<Arc<ApiState>>,
    Json(payload): Json<AuthPayload>,
) -> Result<ApiResponse, ApiError> {
    let new_user = services::auth::register(&state, payload, &[Role::User]).await?;
    // TODO: Turn new_user into a DTO after implementing some sort of DTO system for users.

    ApiResponse::builder()
        .with_success(true)
        .with_code(StatusCode::CREATED)
        .with_api_version(version)
        .with_message("User created")
        .with_payload(json!({ "user": new_user }))
        .build()
        .as_ok()
}

pub fn create_routes(state: Arc<ApiState>) -> Router {
    Router::new()
        .route("/register", post(register))
        .with_state(state)
}
