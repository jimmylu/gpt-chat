use axum::{
    Router,
    response::{Html, IntoResponse},
    routing::get,
};

mod sse;

use chat_core::{Chat, Message};
use futures::StreamExt;
use sqlx::postgres::PgListener;
pub use sse::*;
use tracing::info;

pub enum Event {
    NewChat(Chat),
    AddToChat(Chat),
    RemoveFromChat(Chat),
    NewMessage(Message),
}

pub fn get_router() -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/events", get(sse::sse_handler))
}

async fn index_handler() -> impl IntoResponse {
    let html = include_str!("../index.html");
    Html(html)
}

pub async fn setup_pg_listener() -> anyhow::Result<()> {
    let mut listener =
        PgListener::connect("postgres://jimmylu:jimmylu@localhost:5432/chat").await?;

    listener.listen("chat").await?;
    listener.listen("chat_message_created").await?;

    let mut stream = listener.into_stream();

    tokio::spawn(async move {
        while let Some(Ok(msg)) = stream.next().await {
            info!("{:?}", msg);
        }
    });

    Ok(())
}
