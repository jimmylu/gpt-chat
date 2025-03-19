mod config;
mod error;
mod handlers;
mod middlewares;
mod models;

use core::fmt;
use std::{ops::Deref, sync::Arc};

use anyhow::Context;
use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
use chat_core::{verify_token, DecodingKey, EncodingKey, TokenVerify};
use handlers::*;
use middlewares::verify_chat;
use sqlx::PgPool;
use tokio::fs;

pub use chat_core::User;
pub use config::AppConfig;
pub use error::AppError;
pub use models::*;
pub async fn get_router(config: AppConfig) -> Result<Router, AppError> {
    let state = AppState::try_new(config).await?;

    let chat = Router::new()
        .route(
            "/{id}",
            get(get_chat_handler)
                .post(send_message_handler)
                .patch(update_chat_handler)
                .delete(delete_chat_handler),
        )
        .route("/{id}/messages", get(list_message_handler))
        .layer(from_fn_with_state(state.clone(), verify_chat))
        .route("/", get(list_chat_handler).post(create_chat_handler));

    let api_router = Router::new()
        .nest("/chats", chat)
        .route("/users/{ws_name}", get(user_list_handler))
        .route("/upload", post(upload_handler))
        .route("/files/{ws_id}/{*file_url}", get(download_handler))
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler));

    let app = Router::new()
        .route("/", get(index))
        .nest("/api", api_router)
        .with_state(state);

    Ok(app)
}

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    inner: Arc<AppStateInner>,
}

impl AppState {
    pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
        fs::create_dir_all(&config.server.base_dir)
            .await
            .context("create base dir failed")?;

        let sk = EncodingKey::load(&config.auth.sk).context("load sk failed")?;
        let pk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
        let pool = PgPool::connect(&config.server.db_url)
            .await
            .context("connect to db failed")?;
        Ok(Self {
            inner: Arc::new(AppStateInner {
                config,
                sk,
                pk,
                pool,
            }),
        })
    }
}

impl TokenVerify for AppState {
    type Error = AppError;

    fn verify(&self, token: &str) -> Result<User, Self::Error> {
        self.inner.pk.verify(token).map_err(AppError::from)
    }
}

#[allow(unused)]
struct AppStateInner {
    config: AppConfig,
    sk: EncodingKey,
    pk: DecodingKey,
    pool: PgPool,
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl fmt::Debug for AppStateInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppStateInner")
            .field("config", &self.config)
            .field("pool", &self.pool)
            .finish()
    }
}

#[cfg(test)]
mod test_utils {

    use sqlx::Executor;
    use sqlx_db_tester::TestPg;

    use chat_core::{Chat, Workspace};

    use super::*;

    impl AppState {
        #[allow(unused)]
        pub async fn new_for_test() -> Result<(TestPg, Self), AppError> {
            let config = AppConfig::load()?;

            let sk = EncodingKey::load(&config.auth.sk).context("load sk failed")?;
            let pk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
            let db_url_prefix = config.server.db_url.rfind('/').expect("db_url invalid");
            let db_url = config.server.db_url[..db_url_prefix].to_string();

            let (tdb, pool) = get_pg_and_pool(Some(&db_url)).await;

            let state = Self {
                inner: Arc::new(AppStateInner {
                    config,
                    sk,
                    pk,
                    pool,
                }),
            };
            Ok((tdb, state))
        }
    }

    async fn get_pg_and_pool(url: Option<&str>) -> (TestPg, PgPool) {
        let url = url.unwrap_or("postgres://jimmylu:jimmylu@localhost:5432");
        let tdb = TestPg::new(url.to_string(), std::path::Path::new("../migrations"));
        let pool = tdb.get_pool().await;

        // run prepare sql to insert test data
        let sql = include_str!("../fixtures/test.sql").split(";");
        let mut ts = pool.begin().await.expect("begin tx failed");
        for s in sql {
            if s.trim().is_empty() {
                continue;
            }
            ts.execute(s).await.expect("execute sql failed");
        }
        ts.commit().await.expect("commit tx failed");

        (tdb, pool)
    }

    #[tokio::test]
    async fn test_get_pg_and_pool_should_work() -> Result<(), AppError> {
        let (tdb, pool) = get_pg_and_pool(None).await;
        assert_eq!(
            tdb.server_url(),
            "postgres://jimmylu:jimmylu@localhost:5432"
        );
        let users = sqlx::query_as::<_, User>("select * from users")
            .fetch_all(&pool)
            .await?;
        assert_eq!(users.len(), 10);

        let workspaces = sqlx::query_as::<_, Workspace>("select * from workspaces")
            .fetch_all(&pool)
            .await?;
        assert_eq!(workspaces.len(), 3);

        let chats = sqlx::query_as::<_, Chat>("select * from chats")
            .fetch_all(&pool)
            .await?;
        assert_eq!(chats.len(), 9);

        Ok(())
    }
}
