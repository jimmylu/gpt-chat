use crate::{models::ChatFile, AppError, AppState};

use super::{CreateMessage, Message};

impl AppState {
    #[allow(unused)]
    pub async fn create_message(
        &self,
        input: CreateMessage,
        sender_id: u64,
        chat_id: u64,
    ) -> Result<Message, AppError> {
        //verity content
        if input.content.is_empty() && input.files.is_empty() {
            return Err(AppError::CreateMessageError(
                "Content  can not be empty".to_string(),
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
}
