use axum::{
    Router,
    response::{Html, IntoResponse},
    routing::get,
};

mod sse;

pub use sse::*;

pub fn get_router() -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/events", get(sse::sse_handler))
}

async fn index_handler() -> impl IntoResponse {
    let html = include_str!("../index.html");
    Html(html)
}
