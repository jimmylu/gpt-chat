mod auth;
mod chat;
mod messages;

pub(crate) use auth::*;
use axum::response::IntoResponse;

pub(crate) use chat::*;

pub(crate) async fn index() -> impl IntoResponse {
    "Hello, World!"
}
