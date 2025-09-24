/* src/error.rs */

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Pathmap error: {0}")]
    Pathmap(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Project '{0}' not found")]
    ProjectNotFound(String),

    #[error("Admin operation failed: {0}")]
    AdminOperationFailed(String), // New error for admin operations

    #[error("Path '{0}' not found")]
    NotFound(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Pathmap(err_str) => {
                if err_str.contains("row not found") {
                    (
                        StatusCode::NOT_FOUND,
                        "The requested key or path was not found.".to_string(),
                    )
                } else if err_str.contains("UNIQUE constraint failed") {
                    (
                        StatusCode::CONFLICT,
                        "The key already exists. Use PUT to overwrite.".to_string(),
                    )
                } else {
                    (StatusCode::INTERNAL_SERVER_ERROR, err_str)
                }
            }
            AppError::AdminOperationFailed(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::ProjectNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Io(_) | AppError::SerdeJson(_) | AppError::ConfigError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}
