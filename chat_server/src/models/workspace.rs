use crate::{AppError, AppState};

use super::{Workspace, DEFAULT_OWNER_ID};

impl AppState {
    pub async fn workspace_create(&self, name: &str) -> Result<Workspace, AppError> {
        let workspace: Option<Workspace> =
            sqlx::query_as("select * from workspaces where name = $1")
                .bind(name)
                .fetch_optional(&self.pool)
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
        .fetch_one(&self.pool)
        .await?;
        Ok(workspace)
    }

    #[allow(unused)]
    pub async fn workspace_owner_update(
        &self,
        id: u64,
        owner_id: u64,
    ) -> Result<Workspace, AppError> {
        let workspace = sqlx::query_as(
            r#"
            update workspaces
            set owner_id = $1
            where id = $2
            returning id, name, owner_id, created_at, updated_at
            "#,
        )
        .bind(owner_id as i64)
        .bind(id as i64)
        .fetch_one(&self.pool)
        .await?;
        Ok(workspace)
    }

    pub async fn workspace_fetch_by_name(&self, name: &str) -> Result<Option<Workspace>, AppError> {
        let workspace = sqlx::query_as(
            r#"
            select id, name, owner_id, created_at, updated_at from workspaces where name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        Ok(workspace)
    }
}

#[cfg(test)]
mod tests {
    use crate::models::CreateUserPayload;

    use super::*;

    #[tokio::test]
    async fn test_create_workspace() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let workspace = state.workspace_create("Test Workspace").await?;
        assert_eq!(workspace.name, "Test Workspace");
        Ok(())
    }

    #[tokio::test]
    async fn test_get_by_name() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let ws = state.workspace_fetch_by_name("Test ws").await?;
        assert!(ws.is_some());
        assert_eq!(ws.unwrap().id, 1);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_by_name_should_fail_when_workspace_does_not_exist() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let ws = state.workspace_fetch_by_name("Test ws 3").await?;
        assert!(ws.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn test_update_owner() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let workspace = state.workspace_create("Test Workspace").await?;
        assert_eq!(workspace.owner_id, DEFAULT_OWNER_ID);

        let user = state
            .user_create(CreateUserPayload {
                email: "test@test.com".to_string(),
                fullname: "test".to_string(),
                workspace: "Test Workspace".to_string(),
                password: "test".to_string(),
            })
            .await?;
        assert_eq!(user.ws_id, workspace.id);

        let user_1 = state
            .user_create(CreateUserPayload {
                email: "test1@test.com".to_string(),
                fullname: "test1".to_string(),
                workspace: "Test Workspace".to_string(),
                password: "test".to_string(),
            })
            .await?;
        assert_eq!(user_1.ws_id, workspace.id);

        let workspace = state
            .workspace_owner_update(workspace.id as u64, user_1.id as u64)
            .await?;
        assert_eq!(workspace.owner_id, user_1.id);
        Ok(())
    }
}
