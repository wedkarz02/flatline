use axum::{extract::rejection::PathRejection, http::StatusCode, response::IntoResponse};

use crate::{routes::ApiResponse, services::auth::AuthError};

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("resource not found: {0}")]
    NotFound(String),
    #[error("authentication error: {0}")]
    Auth(#[from] AuthError),
    #[error("internal server error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, msg) = match self {
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ApiError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::Auth(ref auth_err) => (auth_err.status_code(), self.to_string()),
            ApiError::Internal(error) => {
                tracing::error!("{}", error);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "something went wrong".to_owned(),
                )
            }
        };

        ApiResponse::builder()
            .with_success(false)
            .with_code(status)
            .with_message(&msg)
            .build()
            .into_response()
    }
}

impl From<PathRejection> for ApiError {
    fn from(value: PathRejection) -> Self {
        ApiError::BadRequest(format!("path rejection error: {}", value))
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(value: sqlx::Error) -> Self {
        ApiError::Internal(anyhow::Error::new(value))
    }
}

impl From<sqlx::migrate::MigrateError> for ApiError {
    fn from(value: sqlx::migrate::MigrateError) -> Self {
        ApiError::Internal(anyhow::Error::new(value))
    }
}

impl From<jsonwebtoken::errors::Error> for ApiError {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        ApiError::Internal(anyhow::Error::new(value))
    }
}
