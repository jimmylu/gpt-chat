use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

mod user;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: i64,
    pub fullname: String,
    pub email: String,

    #[sqlx(default)]
    #[serde(skip)]
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserPayload {
    pub fullname: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignInPayload {
    pub email: String,
    pub password: String,
}

#[cfg(test)]
impl User {
    pub fn new(id: i64, fullname: String, email: String) -> Self {
        Self {
            id,
            fullname,
            email,
            password_hash: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
