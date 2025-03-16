use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::AppError;

mod chat;
mod file;
mod message;
mod user;
mod workspace;

const DEFAULT_OWNER_ID: i64 = 0;

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

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CreateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
    pub public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "chat_type", rename_all = "snake_case")]
pub enum ChatType {
    Single,
    Group,
    PrivateChannel,
    PublicChannel,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub id: i64,
    pub chat_id: i64,
    pub sender_id: i64,
    pub content: String,
    pub files: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq)]
pub struct ChatUser {
    pub fullname: String,
    pub email: String,
    pub id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserPayload {
    pub fullname: String,
    pub workspace: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignInPayload {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatFile {
    pub ws_id: u64,
    pub ext: String,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessage {
    pub content: String,
    pub files: Vec<String>,
}

#[cfg(test)]
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

impl FromStr for ChatFile {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // file format: /files/0/a94/a8f/e5ccb19ba61c4c0873d391e987982fbbd3.txt
        let Some(s) = s.strip_prefix("/files/") else {
            return Err(AppError::InvalidChatFilePath(format!(
                "Invalid chat file path {}",
                s
            )));
        };

        let parts: Vec<&str> = s.split('/').collect();

        if parts.len() != 4 {
            return Err(AppError::InvalidChatFilePath(format!(
                "Invalid chat file path {}",
                s
            )));
        }

        let Ok(ws_id) = parts[0].parse::<u64>() else {
            return Err(AppError::InvalidChatFilePath(format!(
                "Invalid workspace id {}",
                parts[0]
            )));
        };
        let Some((part3, ext)) = parts[3].split_once('.') else {
            return Err(AppError::InvalidChatFilePath(format!(
                "Invalid chat file path {}",
                s
            )));
        };

        let hash = parts[1].to_owned() + parts[2] + part3;

        Ok(Self {
            ws_id,
            ext: ext.to_string(),
            hash,
        })
    }
}
