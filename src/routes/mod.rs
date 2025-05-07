use std::{collections::HashMap, fmt::Display, str::FromStr, sync::Arc};

use axum::{
    extract::{FromRequestParts, Path},
    http::{request::Parts, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, RequestPartsExt, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;

use crate::{error::AppError, AppState};

pub mod user;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub success: bool,
    pub code: u16,
    pub api_version: String,
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
    api_version: String,
    message: String,
    payload: Option<serde_json::Value>,
}

impl Default for ApiResponseBuilder {
    fn default() -> Self {
        Self {
            success: true,
            code: 200,
            api_version: "v1".to_owned(),
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

    pub fn with_api_version(mut self, api_version: ApiVersion) -> Self {
        self.api_version = api_version.to_string();
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
            api_version: self.api_version,
            message: self.message,
            payload: self.payload,
        }
    }
}

#[derive(Debug)]
pub enum ApiVersion {
    V1,
    V2,
}

impl FromStr for ApiVersion {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "v1" => Ok(Self::V1),
            "v2" => Ok(Self::V2),
            v => Err(AppError::BadRequest(format!(
                "version ({}) not supported",
                v
            ))),
        }
    }
}

impl Display for ApiVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiVersion::V1 => write!(f, "v1"),
            ApiVersion::V2 => write!(f, "v2"),
        }
    }
}

impl<S> FromRequestParts<S> for ApiVersion
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let params: Path<HashMap<String, String>> = parts
            .extract()
            .await
            .map_err(|e| AppError::BadRequest(format!("path rejection error: {}", e)))?;

        let version = params
            .get("version")
            .ok_or_else(|| AppError::BadRequest("version param missing".to_string()))?;

        ApiVersion::from_str(version)
    }
}

pub async fn fallback_handler(req: axum::extract::Request) -> Result<(), AppError> {
    Err(AppError::NotFound(req.uri().to_string()))
}

pub async fn health_check(version: ApiVersion) -> Result<impl IntoResponse, AppError> {
    Ok(ApiResponse::builder()
        .with_success(true)
        .with_code(StatusCode::OK)
        .with_api_version(version)
        .with_message("healthy")
        .build())
}

pub fn create_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .nest(
            "/api/{version}/user",
            user::create_routes(Arc::clone(&state)),
        )
        .route("/api/{version}/health", get(health_check))
        .fallback(fallback_handler)
        .layer(TraceLayer::new_for_http())
}
