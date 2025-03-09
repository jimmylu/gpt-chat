use std::mem;

use crate::{AppError, User};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use sqlx::PgPool;

use super::{CreateUserPayload, Workspace, DEFAULT_OWNER_ID};

impl User {
    // find a user by email
    pub async fn find_by_email(email: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let user = sqlx::query_as(
            "select id, ws_id, fullname, email, created_at, updated_at from users where email = $1",
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;
        Ok(user)
    }

    // verify email and password
    pub async fn verify(
        email: &str,
        password: &str,
        pool: &PgPool,
    ) -> Result<Option<Self>, AppError> {
        let user: Option<Self> = sqlx::query_as(
            "select id, ws_id, fullname, email, password_hash, created_at, updated_at from users where email = $1",
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        match user {
            Some(mut user) => {
                // take the password hash from the user
                let password_hash = mem::take(&mut user.password_hash);
                let is_valid = verify_password(password, &password_hash.unwrap_or_default())?;
                if is_valid {
                    Ok(Some(user))
                } else {
                    Err(AppError::InvalidCredentials)
                }
            }
            None => Ok(None),
        }
    }
    /// 创建用户的方法逻辑：
    /// 1. 检查用户是否已存在，如果存在则返回错误。
    /// 2. 检查工作区是否存在，如果不存在则创建一个新的工作区。
    /// 3. 对用户密码进行哈希处理。
    /// 4. 将用户信息插入到数据库中，并返回创建的用户。
    /// 5. 如果工作区的拥有者是默认拥有者，则更新工作区的拥有者为新创建的用户。
    // TODO: use transaction for workspace creation and user creation
    pub async fn create(input: CreateUserPayload, pool: &PgPool) -> Result<Self, AppError> {
        let user: Option<Self> = sqlx::query_as("select * from users where email = $1")
            .bind(&input.email)
            .fetch_optional(pool)
            .await?;
        if user.is_some() {
            return Err(AppError::UserAlreadyExists);
        }

        // if workspace is not provided, use the default workspace
        let ws = match Workspace::get_by_name(&input.workspace, pool).await? {
            Some(ws) => ws,
            None => Workspace::create(&input.workspace, pool).await?,
        };

        let password_hash = hash_password(&input.password)?;

        let user: Self = sqlx::query_as(
            "insert into users (fullname, email, password_hash, ws_id) values ($1, $2, $3, $4) returning id, ws_id, fullname, email, created_at, updated_at",
        )
        .bind(input.fullname)
        .bind(input.email)
        .bind(password_hash)
        .bind(ws.id)
        .fetch_one(pool)
        .await?;

        // if workspace is not owned by the user, update the owner
        if ws.owner_id == DEFAULT_OWNER_ID {
            ws.update_owner(user.id as u64, pool).await?;
        }

        Ok(user)
    }

    pub async fn add_to_workspace(
        &self,
        workspace_id: i64,
        pool: &PgPool,
    ) -> Result<Self, AppError> {
        let user = sqlx::query_as(
            r#"
            update users
            set ws_id = $1
            where id = $2 and ws_id = 1
            returning id, ws_id, fullname, email, created_at, updated_at
            "#,
        )
        .bind(workspace_id)
        .bind(self.id)
        .fetch_one(pool)
        .await?;
        Ok(user)
    }
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(password_hash)
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(password_hash)?;
    let password_verified = argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok();
    Ok(password_verified)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils;

    #[tokio::test]
    async fn hash_password_and_verify_should_work() -> Result<(), AppError> {
        let password = "test";
        let password_hash = hash_password(password)?;
        assert_eq!(password_hash.len(), 97);
        assert_ne!(password_hash, password);
        let is_valid = verify_password(password, &password_hash)?;
        assert!(is_valid);
        Ok(())
    }

    #[tokio::test]
    async fn create_user_should_work() -> Result<(), AppError> {
        // test by sqlx-db-tester
        let (_tdb, pool) = test_utils::get_pg_and_pool(None).await;

        let email = "test@test.com";
        let fullname = "test";
        let password = "test";

        let user = User::create(
            CreateUserPayload {
                email: email.to_string(),
                fullname: fullname.to_string(),
                workspace: "Default".to_string(),
                password: password.to_string(),
            },
            &pool,
        )
        .await?;
        assert_ne!(user.id, 0);
        assert_eq!(user.email, email);
        assert_eq!(user.fullname, fullname);

        Ok(())
    }

    #[tokio::test]
    async fn create_user_and_update_owner_should_work() -> Result<(), AppError> {
        let (_tdb, pool) = test_utils::get_pg_and_pool(None).await;

        let ws = Workspace::create("Test Workspace", &pool).await?;
        let user = User::create(
            CreateUserPayload {
                email: "test@test.com".to_string(),
                fullname: "test".to_string(),
                workspace: ws.name.clone(),
                password: "test".to_string(),
            },
            &pool,
        )
        .await?;

        let ws = ws.update_owner(user.id as u64, &pool).await?;
        assert_eq!(ws.owner_id, user.id);

        Ok(())
    }

    #[tokio::test]
    async fn verify_user_should_work() -> Result<(), AppError> {
        let (_tdb, pool) = test_utils::get_pg_and_pool(None).await;

        let email = "test@test.com";
        let fullname = "test";
        let password = "test";

        let user = User::create(
            CreateUserPayload {
                email: email.to_string(),
                fullname: fullname.to_string(),
                workspace: "Default".to_string(),
                password: password.to_string(),
            },
            &pool,
        )
        .await?;

        let user_1 = User::verify(email, password, &pool).await?;
        assert!(user_1.is_some());
        assert_eq!(user_1.unwrap().id, user.id);

        Ok(())
    }

    #[tokio::test]
    async fn verify_user_should_fail_when_password_is_incorrect() -> Result<(), AppError> {
        let (_tdb, pool) = test_utils::get_pg_and_pool(None).await;

        let email = "test@test.com";
        let fullname = "test";
        let password = "test";
        let wrong_password = "wrong";

        let _user = User::create(
            CreateUserPayload {
                email: email.to_string(),
                fullname: fullname.to_string(),
                workspace: "Default".to_string(),
                password: password.to_string(),
            },
            &pool,
        )
        .await;

        let result = User::verify(email, wrong_password, &pool).await;
        matches!(result, Err(AppError::InvalidCredentials));

        Ok(())
    }

    #[tokio::test]
    async fn verify_user_should_fail_when_email_is_incorrect() -> Result<(), AppError> {
        let (_tdb, pool) = test_utils::get_pg_and_pool(None).await;

        let email = "test@test.com";
        let password = "test";

        let _user = User::create(
            CreateUserPayload {
                email: email.to_string(),
                fullname: "test".to_string(),
                workspace: "Default".to_string(),
                password: password.to_string(),
            },
            &pool,
        )
        .await?;

        let wrong_email = "wrong@test.com";
        let user_1 = User::verify(wrong_email, password, &pool).await?;
        assert!(user_1.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn find_user_by_email_should_work() -> Result<(), AppError> {
        let (_tdb, pool) = test_utils::get_pg_and_pool(None).await;

        let email = "test@test.com";
        let fullname = "test";
        let password = "test";

        let _user = User::create(
            CreateUserPayload {
                email: email.to_string(),
                fullname: fullname.to_string(),
                workspace: "Default".to_string(),
                password: password.to_string(),
            },
            &pool,
        )
        .await?;

        let user = User::find_by_email(email, &pool).await?;
        assert!(user.is_some());
        assert_eq!(user.clone().unwrap().email, email);
        assert_eq!(user.unwrap().fullname, fullname);

        Ok(())
    }

    #[tokio::test]
    async fn add_to_workspace_should_work() -> Result<(), AppError> {
        let (_tdb, pool) = test_utils::get_pg_and_pool(None).await;

        let user = User::create(
            CreateUserPayload {
                email: "test@test.com".to_string(),
                fullname: "test".to_string(),
                workspace: "Default".to_string(),
                password: "test".to_string(),
            },
            &pool,
        )
        .await?;

        let workspace = Workspace::create("Test Workspace", &pool).await?;
        let user_1 = user.add_to_workspace(workspace.id, &pool).await?;
        assert_eq!(user_1.id, user.id);
        assert_eq!(user_1.ws_id, workspace.id);
        let users = workspace.fetch_all_chat_users(&pool).await?;
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].id, user.id);

        Ok(())
    }
}
