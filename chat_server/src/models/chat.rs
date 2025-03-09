use sqlx::{Pool, Postgres};

use crate::{
    models::{ChatType, ChatUser},
    AppError,
};

use super::{Chat, CreateChat};

impl Chat {
    #[allow(unused)]
    pub async fn create(
        ws_id: u64,
        input: CreateChat,
        pool: &Pool<Postgres>,
    ) -> Result<Chat, AppError> {
        let len = input.members.len();
        if len < 2 {
            return Err(AppError::CreateChatError(
                "At least 2 members are required".to_string(),
            ));
        }
        if len > 8 && input.name.is_none() {
            return Err(AppError::CreateChatError(
                "Group chat with more than 8 members must have a name".to_string(),
            ));
        }
        let users = ChatUser::fetch_by_ids(&input.members, pool).await?;
        if users.len() != len {
            return Err(AppError::CreateChatError(
                "One or more members do not exist".to_string(),
            ));
        }

        let chat_type = match (&input.name, len) {
            (None, 2) => ChatType::Single,
            (None, _) => ChatType::Group,
            (Some(_), _) => {
                if input.public {
                    ChatType::PublicChannel
                } else {
                    ChatType::PrivateChannel
                }
            }
        };
        let chat = sqlx::query_as(
            r#"
            INSERT INTO chats (ws_id, name, type, members)
            VALUES ($1, $2, $3, $4)
            RETURNING *
        "#,
        )
        .bind(ws_id as i64)
        .bind(input.name)
        .bind(chat_type)
        .bind(input.members)
        .fetch_one(pool)
        .await?;

        Ok(chat)
    }

    #[allow(unused)]
    pub async fn fetch_all(ws_id: u64, pool: &Pool<Postgres>) -> Result<Vec<Chat>, AppError> {
        let chats = sqlx::query_as(
            r#"
            SELECT * FROM chats WHERE ws_id = $1
            "#,
        )
        .bind(ws_id as i64)
        .fetch_all(pool)
        .await?;

        Ok(chats)
    }

    #[allow(unused)]
    pub async fn get_by_id(id: i64, pool: &Pool<Postgres>) -> Result<Option<Chat>, AppError> {
        let chat = sqlx::query_as(
            r#"
            SELECT * FROM chats WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(chat)
    }
}

#[cfg(test)]
mod tests {

    use crate::test_utils;

    use super::*;

    #[tokio::test]
    async fn test_create_chat_group_should_work() -> Result<(), AppError> {
        let (_pg, pool) = test_utils::get_pg_and_pool(None).await;
        let chat = Chat::create(
            2,
            CreateChat {
                name: None,
                members: vec![1, 2, 3],
                public: false,
            },
            &pool,
        )
        .await?;
        assert_eq!(chat.r#type, ChatType::Group);
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.name, None);
        assert_eq!(chat.ws_id, 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_create_chat_single_should_work() -> Result<(), AppError> {
        let (_pg, pool) = test_utils::get_pg_and_pool(None).await;
        let chat = Chat::create(
            2,
            CreateChat {
                name: None,
                members: vec![1, 2],
                public: false,
            },
            &pool,
        )
        .await?;
        assert_eq!(chat.r#type, ChatType::Single);
        assert_eq!(chat.members.len(), 2);
        assert_eq!(chat.name, None);
        assert_eq!(chat.ws_id, 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_create_chat_public_channel_should_work() -> Result<(), AppError> {
        let (_pg, pool) = test_utils::get_pg_and_pool(None).await;
        let chat = Chat::create(
            2,
            CreateChat {
                name: Some("Public Channel".to_string()),
                members: vec![1, 2],
                public: true,
            },
            &pool,
        )
        .await?;
        assert_eq!(chat.r#type, ChatType::PublicChannel);
        assert_eq!(chat.members.len(), 2);
        assert_eq!(chat.name, Some("Public Channel".to_string()));
        assert_eq!(chat.ws_id, 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_create_chat_should_fail_when_members_do_not_exist() -> Result<(), AppError> {
        let (_pg, pool) = test_utils::get_pg_and_pool(None).await;
        let chat = Chat::create(
            2,
            CreateChat {
                name: None,
                members: vec![1, 2, 3, 10],
                public: false,
            },
            &pool,
        )
        .await;
        assert!(matches!(
            chat,
            Err(AppError::CreateChatError(msg)) if msg == "One or more members do not exist"
        ));

        Ok(())
    }

    #[tokio::test]
    async fn test_create_chat_should_fail_when_group_chat_has_more_than_8_members(
    ) -> Result<(), AppError> {
        let (_pg, pool) = test_utils::get_pg_and_pool(None).await;
        let chat = Chat::create(
            2,
            CreateChat {
                name: None,
                members: vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                public: false,
            },
            &pool,
        )
        .await;
        assert!(matches!(
            chat,
            Err(AppError::CreateChatError(msg)) if msg == "Group chat with more than 8 members must have a name"
        ));
        Ok(())
    }

    #[tokio::test]
    async fn test_create_chat_should_fail_when_group_chat_has_more_than_8_members_and_name_is_provided(
    ) -> Result<(), AppError> {
        let (_pg, pool) = test_utils::get_pg_and_pool(None).await;
        let chat = Chat::create(
            2,
            CreateChat {
                name: None,
                members: vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                public: false,
            },
            &pool,
        )
        .await;
        assert!(matches!(
            chat,
            Err(AppError::CreateChatError(msg)) if msg == "Group chat with more than 8 members must have a name"
        ));
        Ok(())
    }

    #[tokio::test]
    async fn test_chat_get_by_id_should_work() -> Result<(), AppError> {
        let (_pg, pool) = test_utils::get_pg_and_pool(None).await;

        let chat = Chat::get_by_id(1, &pool).await?;
        assert!(chat.is_some());
        let chat = chat.unwrap();
        assert_eq!(chat.id, 1);
        assert_eq!(chat.r#type, ChatType::PrivateChannel);
        assert_eq!(chat.members.len(), 2);
        assert_eq!(chat.name, Some("Test Chat".to_string()));
        assert_eq!(chat.ws_id, 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_chat_fetch_all_should_work() -> Result<(), AppError> {
        let (_pg, pool) = test_utils::get_pg_and_pool(None).await;
        let chats = Chat::fetch_all(1, &pool).await?;
        assert_eq!(chats.len(), 4);

        let chats = Chat::fetch_all(4, &pool).await?;
        assert_eq!(chats.len(), 0);
        Ok(())
    }
}
