use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json,
};
use config::Config;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, prelude::FromRow, Pool, Postgres};
use uuid::Uuid;

pub mod config;

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct User {
    id: Uuid,
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct CreateUserReq {
    username: String,
    password: String,
}

async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserReq>,
) -> impl IntoResponse {
    let created_user = sqlx::query_as::<_, User>(
        "INSERT INTO users (username, password) VALUES ($1, $2) RETURNING id, username, password",
    )
    .bind(payload.username)
    .bind(payload.password)
    .fetch_one(&state.db)
    .await
    .unwrap();

    Json(created_user)
}

async fn get_all_users(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let users = sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch_all(&state.db)
        .await
        .unwrap();

    Json(users)
}

async fn get_user_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let users = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&state.db)
        .await
        .unwrap();

    Json(users)
}

#[derive(Clone)]
pub struct AppState {
    db: Pool<Postgres>,
}

pub async fn run() -> anyhow::Result<()> {
    let config = Config::parse();
    tracing::info!("Environment configuration loaded: {:?}", config);

    let pool = PgPoolOptions::new()
        .max_connections(config.postgres_pool)
        .connect(&config.postgres_uri())
        .await?;

    sqlx::migrate!()
        .run(&pool)
        .await?;

    let state = Arc::new(AppState { db: pool });

    let app = axum::Router::new()
        .route("/users", get(get_all_users))
        .route("/users", post(create_user))
        .route("/users/{id}", get(get_user_by_id))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(config.socket_addr()).await?;
    tracing::info!("Listening on: {}", listener.local_addr()?);

    axum::serve(listener, app).await?;
    Ok(())
}
