use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Extension, Json, Router,
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    error::ApiError,
    models::user::Role,
    routes::{extractors::ApiVersion, ApiResponse},
    services::{self, auth::AuthError, jwt::Claims},
    ApiState,
};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct AuthPayload {
    pub username: String,
    pub password: String,
}

pub async fn register(
    State(state): State<Arc<ApiState>>,
    version: ApiVersion,
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

pub async fn login(
    State(state): State<Arc<ApiState>>,
    version: ApiVersion,
    Json(payload): Json<AuthPayload>,
) -> Result<ApiResponse, ApiError> {
    let (access_token, refresh_token) = services::auth::login(&state, payload).await?;

    ApiResponse::builder()
        .with_success(true)
        .with_code(StatusCode::OK)
        .with_api_version(version)
        .with_message("Login successful")
        .with_payload(json!({
            "access_token": access_token,
            "refresh_token": refresh_token
        }))
        .build()
        .as_ok()
}

pub async fn protected(
    Extension(claims): Extension<Claims>,
    version: ApiVersion,
) -> Result<ApiResponse, ApiError> {
    ApiResponse::builder()
        .with_code(StatusCode::IM_A_TEAPOT)
        .with_api_version(version)
        .with_message(&format!("Hello, {}", claims.username))
        .build()
        .as_ok()
}

pub async fn admin(
    Extension(claims): Extension<Claims>,
    version: ApiVersion,
) -> Result<ApiResponse, ApiError> {
    if !claims.admin {
        return Err(AuthError::Forbidden.into());
    }

    ApiResponse::builder()
        .with_api_version(version)
        .with_message(&format!("Hello, admin ({})", claims.username))
        .build()
        .as_ok()
}

pub fn create_routes(state: Arc<ApiState>) -> Router {
    let public_routes = Router::new()
        .route("/register", post(register))
        .route("/login", post(login));

    let protected_routes = Router::new()
        .route("/protected", get(protected))
        .route("/admin", get(admin))
        .layer(axum::middleware::from_fn(services::auth::auth_guard))
        .layer(Extension(Arc::clone(&state)));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(state)
}
