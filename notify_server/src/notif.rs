use std::{collections::HashSet, sync::Arc};

use chat_core::{Chat, Message};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgListener;
use tracing::{error, info};

use crate::AppState;

#[derive(Debug, Clone, Serialize)]
pub enum AppEvent {
    NewChat(Chat),
    AddToChat(Chat),
    RemoveFromChat(Chat),
    NewMessage(Message),
}

#[derive(Debug)]
pub struct Notification {
    user_ids: HashSet<u64>,
    event: Arc<AppEvent>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatUpdatedPayload {
    pub op: String,
    pub old: Option<Chat>,
    pub new: Option<Chat>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatMessageCreatedPayload {
    pub message: Message,
    pub members: Vec<u64>,
}

pub async fn setup_pg_listener(state: AppState) -> anyhow::Result<()> {
    let mut listener = PgListener::connect(&state.config.server.db_url).await?;

    listener.listen("chat_updated").await?;
    listener.listen("chat_message_created").await?;

    let mut stream = listener.into_stream();

    tokio::spawn(async move {
        while let Some(Ok(notif)) = stream.next().await {
            info!("received notification: {:?}", notif);

            let user_map: &Arc<
                dashmap::DashMap<u64, tokio::sync::broadcast::Sender<Arc<AppEvent>>>,
            > = &state.users;

            let notif = Notification::load(notif.channel(), notif.payload());
            let Ok(notif) = notif else {
                error!("failed to load notification: {:?}", notif);
                continue;
            };

            notif.user_ids.iter().for_each(|user_id| {
                if let Some(user_channel) = user_map.get(user_id) {
                    let tx = user_channel.value();
                    info!("sending event to user {}: {:?}", user_id, notif.event);
                    if let Err(e) = tx.send(notif.event.clone()) {
                        error!("failed to send event to user {}: {:?}", user_id, e);
                    }
                }
            });
        }
    });

    Ok(())
}

impl Notification {
    pub fn load(r#type: &str, payload: &str) -> anyhow::Result<Self> {
        let notif = match r#type {
            "chat_updated" => {
                let payload = serde_json::from_str::<ChatUpdatedPayload>(payload)?;
                let user_ids = get_effected_users(payload.old.as_ref(), payload.new.as_ref());
                Notification {
                    user_ids,
                    event: {
                        match payload.op.as_str() {
                            "INSERT" => Arc::new(AppEvent::NewChat(
                                payload.new.expect("new chat is required"),
                            )),
                            "UPDATE" => Arc::new(AppEvent::AddToChat(
                                payload.new.expect("new chat is required"),
                            )),
                            "DELETE" => Arc::new(AppEvent::RemoveFromChat(
                                payload.old.expect("old chat is required"),
                            )),
                            _ => panic!("Unknown operation: {}", payload.op),
                        }
                    },
                }
            }
            "chat_message_created" => {
                let payload = serde_json::from_str::<ChatMessageCreatedPayload>(payload)?;
                Notification {
                    user_ids: HashSet::from_iter(payload.members),
                    event: Arc::new(AppEvent::NewMessage(payload.message)),
                }
            }
            _ => panic!("Unknown notification type: {}", r#type),
        };

        Ok(notif)
    }
}

fn get_effected_users(old: Option<&Chat>, new: Option<&Chat>) -> HashSet<u64> {
    match (old, new) {
        (Some(old), Some(new)) => {
            // if the chat is the same, no need to notify anyone
            if old.members == new.members {
                return HashSet::new();
            }
            // if the chat is different, notify all the users
            let mut user_ids = HashSet::from_iter(old.members.iter().map(|id| *id as u64));
            user_ids.extend(new.members.iter().map(|id| *id as u64));
            user_ids
        }
        // if the chat is new, notify all the users
        (None, Some(new)) => HashSet::from_iter(new.members.iter().map(|id| *id as u64)),
        // if the chat is deleted, notify all the users
        (Some(old), None) => HashSet::from_iter(old.members.iter().map(|id| *id as u64)),
        // if the chat is not found, notify no one
        (None, None) => HashSet::new(),
    }
}
