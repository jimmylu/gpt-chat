use axum::{Json, response::IntoResponse};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("jwt encoding key error: {0}")]
    JwtError(#[from] jwt_simple::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match &self {
            AppError::JwtError(_) => axum::http::StatusCode::FORBIDDEN,
            AppError::IoError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(json!({
            "error": self.to_string(),
        }));
        (status_code, body).into_response()
    }
}
