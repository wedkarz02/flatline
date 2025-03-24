use std::sync::Arc;

use axum::{http::StatusCode, response::IntoResponse, Json, Router};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;

use crate::{error::AppError, AppState};

pub mod user;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub success: bool,
    pub code: u16,
    pub message: String,
    pub payload: Option<serde_json::Value>,
}

impl ApiResponse {
    pub fn builder() -> ApiResponseBuilder {
        ApiResponseBuilder::default()
    }

    pub fn status_code(&self) -> StatusCode {
        StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl IntoResponse for ApiResponse {
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), Json(self)).into_response()
    }
}

pub struct ApiResponseBuilder {
    success: bool,
    code: u16,
    message: String,
    payload: Option<serde_json::Value>,
}

impl Default for ApiResponseBuilder {
    fn default() -> Self {
        Self {
            success: true,
            code: 200,
            message: Default::default(),
            payload: Default::default(),
        }
    }
}

impl ApiResponseBuilder {
    pub fn with_success(mut self, success: bool) -> Self {
        self.success = success;
        self
    }

    pub fn with_code(mut self, code: StatusCode) -> Self {
        self.code = code.as_u16();
        self
    }

    pub fn with_message(mut self, message: &str) -> Self {
        self.message = String::from(message);
        self
    }

    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn build(self) -> ApiResponse {
        ApiResponse {
            success: self.success,
            code: self.code,
            message: self.message,
            payload: self.payload,
        }
    }
}

pub async fn fallback_handler(req: axum::extract::Request) -> Result<(), AppError> {
    Err(AppError::NotFound(req.uri().to_string()))
}

pub fn create_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .nest("/api/v1/user", user::create_routes(Arc::clone(&state)))
        .fallback(fallback_handler)
        .layer(TraceLayer::new_for_http())
}
