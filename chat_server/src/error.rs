use axum::{response::IntoResponse, Json};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("sql error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("password hash error: {0}")]
    PasswordHashError(#[from] argon2::password_hash::Error),

    #[error("password is incorrect")]
    InvalidCredentials,

    #[error("jwt encoding key error: {0}")]
    JwtError(#[from] jwt_simple::Error),

    #[error("user not found")]
    UserNotFound,

    #[error("http header parse error: {0}")]
    HttpHeaderError(#[from] axum::http::header::InvalidHeaderValue),

    #[error("user already exists")]
    UserAlreadyExists,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match &self {
            AppError::SqlxError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::PasswordHashError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::InvalidCredentials => axum::http::StatusCode::UNAUTHORIZED,
            AppError::JwtError(_) => axum::http::StatusCode::FORBIDDEN,
            AppError::UserNotFound => axum::http::StatusCode::NOT_FOUND,
            AppError::HttpHeaderError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::UserAlreadyExists => axum::http::StatusCode::FORBIDDEN,
        };

        let body = Json(json!({
            "error": self.to_string(),
        }));
        (status_code, body).into_response()
    }
}
