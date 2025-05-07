use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{error::AppError, AppState};

use super::{ApiResponse, ApiVersion};

#[derive(Deserialize)]
struct CreateUserReq {
    username: String,
    password: String,
}

async fn create_user(
    version: ApiVersion,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserReq>,
) -> Result<impl IntoResponse, AppError> {
    let created_user = state
        .db
        .users()
        .create(&payload.username, &payload.password)
        .await?;

    Ok(ApiResponse::builder()
        .with_code(StatusCode::CREATED)
        .with_api_version(version)
        .with_message("user created")
        .with_payload(serde_json::json!({ "user": created_user }))
        .build())
}

async fn get_all_users(
    version: ApiVersion,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let users = state.db.users().find_all().await?;
    Ok(ApiResponse::builder()
        .with_api_version(version)
        .with_message(&format!("found {} users", users.len()))
        .with_payload(serde_json::json!({ "users": users }))
        .build())
}

async fn get_user_by_id(
    version: ApiVersion,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let user = state.db.users().find_by_id(id).await?;
    Ok(ApiResponse::builder()
        .with_api_version(version)
        .with_message("user found")
        .with_payload(serde_json::json!({ "user": user }))
        .build())
}

async fn delete_all_users(
    version: ApiVersion,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let deleted_count = state.db.users().delete_all().await?;
    Ok(ApiResponse::builder()
        .with_api_version(version)
        .with_message("deleted all users")
        .with_payload(serde_json::json!({ "deleted_count": deleted_count }))
        .build())
}

pub fn create_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(get_all_users))
        .route("/", post(create_user))
        .route("/{id}", get(get_user_by_id))
        .route("/", delete(delete_all_users))
        .with_state(state)
}
