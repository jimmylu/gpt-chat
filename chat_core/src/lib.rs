use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

mod middlewares;
mod utils;

pub use middlewares::*;
pub use utils::*;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: i64,
    pub ws_id: i64,
    pub fullname: String,
    pub email: String,

    #[sqlx(default)]
    #[serde(skip)]
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq)]
pub struct Workspace {
    pub id: i64,
    pub name: String,
    pub owner_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq)]
pub struct Chat {
    pub id: i64,
    pub name: Option<String>,
    pub r#type: ChatType,
    pub ws_id: i64,
    pub members: Vec<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "chat_type", rename_all = "snake_case")]
#[serde(rename_all = "camelCase")]
pub enum ChatType {
    #[serde(alias = "single", alias = "Single")]
    Single,
    #[serde(alias = "group", alias = "Group")]
    Group,
    #[serde(alias = "private_channel", alias = "privateChannel")]
    PrivateChannel,
    #[serde(alias = "public_channel", alias = "publicChannel")]
    PublicChannel,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub id: i64,
    pub chat_id: i64,
    pub sender_id: i64,
    pub content: Option<String>,
    pub files: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new(id: i64, ws_id: i64, fullname: String, email: String) -> Self {
        Self {
            id,
            ws_id,
            fullname,
            email,
            password_hash: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
