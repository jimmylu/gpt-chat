use sqlx::PgPool;

use crate::AppError;

use super::{User, Workspace, DEFAULT_OWNER_ID};

impl Workspace {
    pub async fn create(name: &str, pool: &PgPool) -> Result<Self, AppError> {
        let workspace: Option<Self> = sqlx::query_as("select * from workspaces where name = $1")
            .bind(name)
            .fetch_optional(pool)
            .await?;
        if workspace.is_some() {
            return Err(AppError::WorkspaceAlreadyExists);
        }

        let workspace = sqlx::query_as(
            r#"
            insert into workspaces (name, owner_id) values ($1, $2) returning id, name, owner_id, created_at, updated_at
            "#,
        )
        .bind(name)
        .bind(DEFAULT_OWNER_ID)
        .fetch_one(pool)
        .await?;
        Ok(workspace)
    }

    #[allow(unused)]
    pub async fn update_owner(&self, owner_id: u64, pool: &PgPool) -> Result<Self, AppError> {
        let workspace = sqlx::query_as(
            r#"
            update workspaces
            set owner_id = $1
            where id = $2
            returning id, name, owner_id, created_at, updated_at
            "#,
        )
        .bind(owner_id as i64)
        .bind(self.id)
        .fetch_one(pool)
        .await?;
        Ok(workspace)
    }

    #[allow(unused)]
    pub async fn fetch_all_chat_users(&self, pool: &PgPool) -> Result<Vec<User>, AppError> {
        let users = sqlx::query_as(
            r#"
            select id, ws_id, fullname, email, created_at, updated_at from users where ws_id = $1
            "#,
        )
        .bind(self.id)
        .fetch_all(pool)
        .await?;
        Ok(users)
    }

    pub async fn get_by_name(name: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let workspace = sqlx::query_as(
            r#"
            select id, name, owner_id, created_at, updated_at from workspaces where name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(pool)
        .await?;
        Ok(workspace)
    }
}

#[cfg(test)]
mod tests {
    use crate::{models::CreateUserPayload, AppState};

    use super::*;

    #[tokio::test]
    async fn test_create_workspace() -> Result<(), AppError> {
        let (pg, _state) = AppState::new_for_test().await?;
        let pg_pool = pg.get_pool().await;
        let workspace = Workspace::create("Test Workspace", &pg_pool).await?;
        assert_eq!(workspace.name, "Test Workspace");
        Ok(())
    }

    #[tokio::test]
    async fn test_get_by_name() -> Result<(), AppError> {
        let (pg, _state) = AppState::new_for_test().await?;
        let pg_pool = pg.get_pool().await;
        let workspace = Workspace::create("Test Workspace", &pg_pool).await?;
        let workspace_1 = Workspace::get_by_name("Test Workspace", &pg_pool).await?;
        assert!(workspace_1.is_some());
        assert_eq!(workspace_1.unwrap().id, workspace.id);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_by_name_should_fail_when_workspace_does_not_exist() -> Result<(), AppError> {
        let (pg, _state) = AppState::new_for_test().await?;
        let pg_pool = pg.get_pool().await;
        let workspace = Workspace::get_by_name("Test Workspace", &pg_pool).await?;
        assert!(workspace.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn test_fetch_all_chat_users() -> Result<(), AppError> {
        let (pg, _state) = AppState::new_for_test().await?;
        let pg_pool = pg.get_pool().await;
        let workspace = Workspace::create("Test Workspace", &pg_pool).await?;
        let users = workspace.fetch_all_chat_users(&pg_pool).await?;
        assert_eq!(users.len(), 0);

        let user = User::create(
            CreateUserPayload {
                email: "test@test.com".to_string(),
                fullname: "test".to_string(),
                workspace: "Test Workspace".to_string(),
                password: "test".to_string(),
            },
            &pg_pool,
        )
        .await?;
        let user_1 = User::create(
            CreateUserPayload {
                email: "test1@test.com".to_string(),
                fullname: "test1".to_string(),
                workspace: "Test Workspace".to_string(),
                password: "test".to_string(),
            },
            &pg_pool,
        )
        .await?;
        let users = workspace.fetch_all_chat_users(&pg_pool).await?;
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].id, user.id);
        assert_eq!(users[1].id, user_1.id);
        assert_eq!(users[0].ws_id, workspace.id);
        assert_eq!(users[1].ws_id, workspace.id);
        let ws = Workspace::get_by_name("Test Workspace", &pg_pool).await?;
        assert_eq!(ws.unwrap().owner_id, user.id);
        Ok(())
    }

    #[tokio::test]
    async fn test_update_owner() -> Result<(), AppError> {
        let (pg, _state) = AppState::new_for_test().await?;
        let pg_pool = pg.get_pool().await;
        let workspace = Workspace::create("Test Workspace", &pg_pool).await?;
        assert_eq!(workspace.owner_id, DEFAULT_OWNER_ID);

        let user = User::create(
            CreateUserPayload {
                email: "test@test.com".to_string(),
                fullname: "test".to_string(),
                workspace: "Test Workspace".to_string(),
                password: "test".to_string(),
            },
            &pg_pool,
        )
        .await?;
        assert_eq!(user.ws_id, workspace.id);

        let user_1 = User::create(
            CreateUserPayload {
                email: "test1@test.com".to_string(),
                fullname: "test1".to_string(),
                workspace: "Test Workspace".to_string(),
                password: "test".to_string(),
            },
            &pg_pool,
        )
        .await?;
        assert_eq!(user_1.ws_id, workspace.id);

        let workspace = workspace.update_owner(user_1.id as u64, &pg_pool).await?;
        assert_eq!(workspace.owner_id, user_1.id);
        Ok(())
    }
}
