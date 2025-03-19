use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::AppError;

mod chat;
mod file;
mod message;
mod user;
mod workspace;

pub use chat::*;
pub use message::*;
pub use user::*;

const DEFAULT_OWNER_ID: i64 = 0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatFile {
    pub ws_id: u64,
    pub ext: String,
    pub hash: String,
}

impl FromStr for ChatFile {
    type Err = AppError;

    fn from_str(file_url: &str) -> Result<Self, Self::Err> {
        // file format: /files/0/a94/a8f/e5ccb19ba61c4c0873d391e987982fbbd3.txt
        let Some(s) = file_url.strip_prefix("/files/") else {
            return Err(AppError::InvalidChatFilePath(format!(
                "Invalid chat file path {}",
                file_url
            )));
        };

        let parts: Vec<&str> = s.split('/').collect();

        if parts.len() != 4 {
            return Err(AppError::InvalidChatFilePath(format!(
                "Invalid chat file path {}",
                file_url
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
                file_url
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
