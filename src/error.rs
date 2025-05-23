use axum::{http::StatusCode, response::IntoResponse};

use crate::routes::ApiResponse;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("resource not found: {0}")]
    NotFound(String),
    #[error("internal server error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, msg) = match self {
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ApiError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
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
