use std::{convert::Infallible, time::Duration};

use axum::{
    Extension,
    extract::State,
    response::sse::{Event, Sse},
};
use chat_core::User;
use futures::Stream;
use tokio::sync::broadcast;
use tokio_stream::{StreamExt as _, wrappers::BroadcastStream};
use tracing::info;

use crate::{AppEvent, AppState};

const CAPACITY: usize = 256;

pub async fn sse_handler(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    // TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let user_id = user.id as u64;
    let users = &state.users;

    let rx = if let Some(tx) = users.get(&user_id) {
        tx.subscribe()
    } else {
        let (tx, rx) = broadcast::channel(CAPACITY);
        users.insert(user_id, tx);
        rx
    };

    info!("`User {}` subscribed", user_id);

    let stream = BroadcastStream::new(rx).filter_map(|e| e.ok()).map(|e| {
        let name = match e.as_ref() {
            AppEvent::NewChat(_) => "new_chat",
            AppEvent::AddToChat(_) => "add_to_chat",
            AppEvent::RemoveFromChat(_) => "remove_from_chat",
            AppEvent::NewMessage(_) => "new_message",
        };

        let v = serde_json::to_string(&e).expect("failed to serialize event");
        dbg!(format!("Sending event {}: {:?}", name, v));
        Ok(Event::default().data(v).event(name))
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}
