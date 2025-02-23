use std::mem;

use crate::{AppError, User};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use sqlx::PgPool;

impl User {
    // find a user by email
    pub async fn find_by_email(email: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let user = sqlx::query_as(
            "select id, fullname, email, created_at, updated_at from users where email = $1",
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
            "select id, fullname, email, password_hash, created_at, updated_at from users where email = $1",
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

    // create a new user
    pub async fn create(
        email: &str,
        fullname: &str,
        password: &str,
        pool: &PgPool,
    ) -> Result<Self, AppError> {
        let password_hash = hash_password(password)?;
        let user = sqlx::query_as("insert into users (fullname, email, password_hash) values ($1, $2, $3) returning id, fullname, email, created_at, updated_at")
            .bind(fullname)
            .bind(email)
            .bind(password_hash)
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
    use sqlx_db_tester::TestPg;

    use super::*;

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
        let tdb = TestPg::new(
            "postgres://jimmylu:@localhost:5432".to_string(),
            std::path::Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;

        let email = "test@test.com";
        let fullname = "test";
        let password = "test";

        let user = User::create(email, fullname, password, &pool).await?;
        assert_ne!(user.id, 0);
        assert_eq!(user.email, email);
        assert_eq!(user.fullname, fullname);

        Ok(())
    }

    #[tokio::test]
    async fn verify_user_should_work() -> Result<(), AppError> {
        let tdb = TestPg::new(
            "postgres://jimmylu:@localhost:5432".to_string(),
            std::path::Path::new("../migrations"),
        );

        let pool = tdb.get_pool().await;

        let email = "test@test.com";
        let fullname = "test";
        let password = "test";

        let user = User::create(email, fullname, password, &pool).await?;

        let user_1 = User::verify(email, password, &pool).await?;
        assert!(user_1.is_some());
        assert_eq!(user_1.unwrap().id, user.id);

        Ok(())
    }

    #[tokio::test]
    async fn verify_user_should_fail_when_password_is_incorrect() -> Result<(), AppError> {
        let tdb = TestPg::new(
            "postgres://jimmylu:@localhost:5432".to_string(),
            std::path::Path::new("../migrations"),
        );

        let pool = tdb.get_pool().await;

        let email = "test@test.com";
        let fullname = "test";
        let password = "test";
        let wrong_password = "wrong";

        let _user = User::create(email, fullname, password, &pool).await;

        let result = User::verify(email, wrong_password, &pool).await;
        matches!(result, Err(AppError::InvalidCredentials));

        Ok(())
    }

    #[tokio::test]
    async fn verify_user_should_fail_when_email_is_incorrect() -> Result<(), AppError> {
        let tdb = TestPg::new(
            "postgres://jimmylu:@localhost:5432".to_string(),
            std::path::Path::new("../migrations"),
        );

        let email = "test@test.com";
        let password = "test";

        let pool = tdb.get_pool().await;
        let _user = User::create(email, "test", password, &pool).await?;

        let wrong_email = "wrong@test.com";
        let user_1 = User::verify(wrong_email, password, &pool).await?;
        assert!(user_1.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn find_user_by_email_should_work() -> Result<(), AppError> {
        let tdb = TestPg::new(
            "postgres://jimmylu:@localhost:5432".to_string(),
            std::path::Path::new("../migrations"),
        );

        let pool = tdb.get_pool().await;

        let email = "test@test.com";
        let fullname = "test";
        let password = "test";

        let _user = User::create(email, fullname, password, &pool).await?;

        let user = User::find_by_email(email, &pool).await?;
        assert!(user.is_some());
        assert_eq!(user.clone().unwrap().email, email);
        assert_eq!(user.unwrap().fullname, fullname);

        Ok(())
    }
}
