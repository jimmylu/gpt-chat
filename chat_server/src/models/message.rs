use crate::{models::ChatFile, AppError, AppState};

use chat_core::Message;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessage {
    pub content: String,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListMessages {
    pub last_id: Option<u64>,
    pub limit: u64,
}

impl AppState {
    #[allow(unused)]
    pub async fn message_create(
        &self,
        input: CreateMessage,
        sender_id: u64,
        chat_id: u64,
    ) -> Result<Message, AppError> {
        //verity content
        if input.content.is_empty() && input.files.is_empty() {
            return Err(AppError::CreateMessageError(
                "Content or files can not be empty".to_string(),
            ));
        }
        let base_dir = &self.config.server.base_dir;

        // verify files
        for file in &input.files {
            let file: ChatFile = file.parse()?;
            let file_path = file.path(base_dir);
            if !file_path.exists() {
                return Err(AppError::CreateMessageError(format!(
                    "File not exists: {}",
                    file.url()
                )));
            }
        }

        let message = sqlx::query_as(
            r#"
            INSERT INTO messages (content, files, chat_id, sender_id)
            VALUES ($1, $2, $3, $4)
            RETURNING *
        "#,
        )
        .bind(input.content)
        .bind(&input.files)
        .bind(chat_id as i64)
        .bind(sender_id as i64)
        .fetch_one(&self.pool)
        .await?;

        Ok(message)
    }

    #[allow(unused)]
    pub async fn message_list(
        &self,
        input: ListMessages,
        chat_id: u64,
    ) -> Result<Vec<Message>, AppError> {
        let last_id = input.last_id.unwrap_or(i64::MAX as _);
        let messages = sqlx::query_as(
            r#"
            SELECT * FROM messages
            WHERE chat_id = $1
            AND id < $2
            ORDER BY id DESC
            LIMIT $3
        "#,
        )
        .bind(chat_id as i64)
        .bind(last_id as i64)
        .bind(input.limit as i64)
        .fetch_all(&self.pool)
        .await?;
        Ok(messages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_message_create_should_work() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let message = state
            .message_create(
                CreateMessage {
                    content: "Hello, world!".to_string(),
                    files: vec![],
                },
                1,
                1,
            )
            .await?;
        assert_eq!(message.id, 13);
        assert_eq!(message.content, Some("Hello, world!".to_string()));
        Ok(())
    }

    #[tokio::test]
    async fn test_message_create_should_fail_if_content_and_files_are_empty() -> Result<(), AppError>
    {
        let (_tdb, state) = AppState::new_for_test().await?;
        let result = state
            .message_create(
                CreateMessage {
                    content: "".to_string(),
                    files: vec![],
                },
                1,
                1,
            )
            .await;
        assert!(
            matches!(result, Err(AppError::CreateMessageError(msg)) if msg == "Content or files can not be empty")
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_message_create_should_fail_if_file_url_is_invalid() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let result = state
            .message_create(
                CreateMessage {
                    content: "Hello, world!".to_string(),
                    files: vec!["https://example.com/file.txt".to_string()],
                },
                1,
                1,
            )
            .await;
        assert!(
            matches!(result, Err(AppError::InvalidChatFilePath(msg)) if msg == format!("Invalid chat file path {}", "https://example.com/file.txt"))
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_message_create_should_fail_if_file_url_workspace_id_is_invalid(
    ) -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let result = state
            .message_create(
                CreateMessage {
                    content: "Hello, world!".to_string(),
                    files: vec![
                        "/files/a/a94/a8f/e5ccb19ba61c4c0873d391e987982fbbd3.txt".to_string()
                    ],
                },
                1,
                1,
            )
            .await;
        assert!(
            matches!(result, Err(AppError::InvalidChatFilePath(msg)) if msg == format!("Invalid workspace id {}", "a"))
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_message_create_should_work_with_file() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let file_url = upload_dummy_file(&state).await?;
        let message = state
            .message_create(
                CreateMessage {
                    content: "Hello, world!".to_string(),
                    files: vec![file_url.clone()],
                },
                1,
                1,
            )
            .await?;
        assert_eq!(message.id, 13);
        assert_eq!(message.content, Some("Hello, world!".to_string()));
        assert_eq!(message.files.clone().unwrap().len(), 1);
        assert_eq!(
            message.files.clone().unwrap()[0],
            "/files/1/2aa/e6c/35c94fcfb415dbe95f408b9ce91ee846ed.txt"
        );
        Ok(())
    }

    async fn upload_dummy_file(state: &AppState) -> Result<String, AppError> {
        let base_dir = &state.config.server.base_dir;
        let file = ChatFile::new(1, "test.txt".to_string(), b"hello world");
        let file_path = file.path(base_dir);
        std::fs::create_dir_all(file_path.parent().unwrap())?;
        std::fs::write(file_path, b"hello world")?;
        Ok(file.url())
    }

    #[tokio::test]
    async fn test_message_list_should_work() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;

        let messages = state
            .message_list(
                ListMessages {
                    last_id: None,
                    limit: 10,
                },
                1,
            )
            .await?;
        assert_eq!(messages.len(), 10);
        assert_eq!(messages[0].id, 12);
        assert_eq!(messages[0].content, Some("Hello, world12!".to_string()));

        let last_id = messages[9].id;

        let messages = state
            .message_list(
                ListMessages {
                    last_id: Some(last_id as u64),
                    limit: 10,
                },
                1,
            )
            .await?;
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].id, 2);
        assert_eq!(messages[0].content, Some("Hello, world2!".to_string()));
        Ok(())
    }
}
