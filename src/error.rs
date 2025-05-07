use axum::{http::StatusCode, response::IntoResponse};

use crate::routes::ApiResponse;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("resource not found: {0}")]
    NotFound(String),
    #[error("internal server error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, msg) = match self {
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Internal(error) => {
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
            // FIXME: This always defaults to 'v1', fix to get a real version here.
            // .with_api_version(???)
            .with_message(&msg)
            .build()
            .into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        AppError::Internal(anyhow::Error::new(value))
    }
}

impl From<sqlx::migrate::MigrateError> for AppError {
    fn from(value: sqlx::migrate::MigrateError) -> Self {
        AppError::Internal(anyhow::Error::new(value))
    }
}
