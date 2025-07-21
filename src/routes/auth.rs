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
    models::user::{Role, UserDto},
    routes::{extractors::ApiVersion, ApiResponse},
    services::{self, auth::AuthError, jwt::Claims},
    ApiState,
};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct AuthPayload {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RefreshPayload {
    pub refresh_token: String,
}

async fn register(
    State(state): State<Arc<ApiState>>,
    version: ApiVersion,
    Json(payload): Json<AuthPayload>,
) -> Result<ApiResponse, ApiError> {
    let new_user = services::users::create_user(&state, payload, &[Role::User]).await?;
    let user_dto = UserDto::from(&new_user);

    ApiResponse::builder()
        .with_success(true)
        .with_code(StatusCode::CREATED)
        .with_api_version(version)
        .with_message("User created")
        .with_payload(json!({ "user": user_dto }))
        .build()
        .as_ok()
}

async fn login(
    State(state): State<Arc<ApiState>>,
    version: ApiVersion,
    Json(payload): Json<AuthPayload>,
) -> Result<ApiResponse, ApiError> {
    let (access_token, refresh_token, deleted_token) =
        services::auth::login(&state, payload).await?;

    let msg = if let Some(token) = deleted_token {
        format!(
            "Login successful. Oldest session ({}) revoked due to user session limit.",
            token.jti
        )
    } else {
        String::from("Login successful.")
    };

    ApiResponse::builder()
        .with_success(true)
        .with_code(StatusCode::OK)
        .with_api_version(version)
        .with_message(&msg)
        .with_payload(json!({
            "access_token": access_token,
            "refresh_token": refresh_token,
            "token_type": "Bearer",
        }))
        .build()
        .as_ok()
}

async fn logout(
    State(state): State<Arc<ApiState>>,
    version: ApiVersion,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<RefreshPayload>,
) -> Result<ApiResponse, ApiError> {
    let res = services::auth::logout(&state, &payload.refresh_token, &claims.jti).await?;
    let mut builder = ApiResponse::builder().with_api_version(version);

    builder = if let Some(jti) = res {
        builder
            .with_message("Session revoked")
            .with_payload(serde_json::json!({ "session_id": jti }))
    } else {
        builder
            .with_success(false)
            .with_code(StatusCode::NOT_FOUND)
            .with_message("Session not found")
    };

    builder.build().as_ok()
}

async fn protected(
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

async fn admin(
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
        .route("/logout", post(logout))
        .route("/protected", get(protected))
        .route("/admin", get(admin))
        .layer(axum::middleware::from_fn(services::auth::auth_guard))
        .layer(Extension(Arc::clone(&state)));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(state)
}
