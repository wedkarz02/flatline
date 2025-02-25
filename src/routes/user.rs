use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{error::AppError, AppState};

#[derive(Deserialize)]
struct CreateUserReq {
    username: String,
    password: String,
}

async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserReq>,
) -> Result<impl IntoResponse, AppError> {
    let created_user = state
        .db
        .users()
        .create(&payload.username, &payload.password)
        .await?;

    Ok(Json(created_user))
}

async fn get_all_users(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, AppError> {
    let users = state
        .db
        .users()
        .find_all()
        .await?;

    Ok(Json(users))
}

async fn get_user_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let user = state
        .db
        .users()
        .find_by_id(id)
        .await?;

    Ok(Json(user))
}

async fn delete_all_users(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let deleted_count = state
        .db
        .users()
        .delete_all()
        .await?;

    Ok(Json(deleted_count))
}

pub fn create_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(get_all_users))
        .route("/", post(create_user))
        .route("/{id}", get(get_user_by_id))
        .route("/", delete(delete_all_users))
        .with_state(state)
}
