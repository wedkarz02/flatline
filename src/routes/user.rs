use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::AppState;

#[derive(Deserialize)]
struct CreateUserReq {
    username: String,
    password: String,
}

async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserReq>,
) -> impl IntoResponse {
    let created_user = state
        .db
        .users()
        .create_user(&payload.username, &payload.password)
        .await
        .unwrap();

    Json(created_user)
}

async fn get_all_users(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // let users = sqlx::query_as::<_, User>("SELECT * FROM users")
    //     .fetch_all(
    //         &state
    //             .db
    //             .pool,
    //     )
    //     .await
    //     .unwrap();

    let users = state
        .db
        .users()
        .get_all_users()
        .await
        .unwrap();

    Json(users)
}

async fn get_user_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
    //     .bind(id)
    //     .fetch_one(
    //         &state
    //             .db
    //             .pool,
    //     )
    //     .await
    //     .unwrap();

    let user = state
        .db
        .users()
        .get_user_by_id(id)
        .await
        .unwrap();

    Json(user)
}

pub fn create_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(get_all_users))
        .route("/", post(create_user))
        .route("/{id}", get(get_user_by_id))
        .with_state(state)
}
