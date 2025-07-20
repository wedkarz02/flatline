use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};

use crate::{
    error::ApiError,
    models::user::{Role, UserDto},
    routes::{
        auth::AuthPayload,
        extractors::{ApiVersion, VerIdParams},
    },
    services, ApiState,
};

use super::ApiResponse;

async fn create_user(
    State(state): State<Arc<ApiState>>,
    version: ApiVersion,
    Json(payload): Json<AuthPayload>,
) -> Result<ApiResponse, ApiError> {
    let new_user = services::users::create_user(&state, payload, &[Role::User]).await?;
    let user_dto = UserDto::from(new_user);

    ApiResponse::builder()
        .with_code(StatusCode::CREATED)
        .with_api_version(version)
        .with_message("user created")
        .with_payload(serde_json::json!({ "user": user_dto }))
        .build()
        .as_ok()
}

async fn get_all_users(
    State(state): State<Arc<ApiState>>,
    version: ApiVersion,
) -> Result<ApiResponse, ApiError> {
    let users: Vec<UserDto> = state
        .db
        .users()
        .find_all()
        .await?
        .iter()
        .map(UserDto::from)
        .collect();

    ApiResponse::builder()
        .with_api_version(version)
        .with_message(&format!("found {} users", users.len()))
        .with_payload(serde_json::json!({ "users": users }))
        .build()
        .as_ok()
}

async fn get_user_by_id(
    State(state): State<Arc<ApiState>>,
    VerIdParams { version, id }: VerIdParams,
) -> Result<ApiResponse, ApiError> {
    let user = state.db.users().find_by_id(id).await?;
    if user.is_none() {
        return ApiResponse::builder()
            .with_api_version(version)
            .with_message("user not found")
            .with_code(StatusCode::NOT_FOUND)
            .build()
            .as_ok();
    }

    let user_dto = UserDto::from(&user.unwrap());

    ApiResponse::builder()
        .with_api_version(version)
        .with_message("user found")
        .with_payload(serde_json::json!({ "user": user_dto }))
        .build()
        .as_ok()
}

async fn delete_all_users(
    State(state): State<Arc<ApiState>>,
    version: ApiVersion,
) -> Result<ApiResponse, ApiError> {
    let deleted_count = state.db.users().delete_all().await?;
    ApiResponse::builder()
        .with_api_version(version)
        .with_message("deleted all users")
        .with_payload(serde_json::json!({ "deleted_count": deleted_count }))
        .build()
        .as_ok()
}

pub fn create_routes(state: Arc<ApiState>) -> Router {
    Router::new()
        .route("/", get(get_all_users))
        .route("/", post(create_user))
        .route("/{id}", get(get_user_by_id))
        .route("/", delete(delete_all_users))
        .with_state(state)
}
