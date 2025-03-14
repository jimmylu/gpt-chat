mod config;
mod error;
mod handlers;
mod middlewares;
mod models;
mod utils;

use core::fmt;
use std::{ops::Deref, sync::Arc};

use anyhow::Context;
use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
use handlers::*;
use middlewares::verify_token;
use sqlx::PgPool;
use tokio::fs;
use utils::DecodingKey;
use utils::EncodingKey;

pub use config::AppConfig;
pub use error::AppError;
pub use models::User;

pub async fn get_router(config: AppConfig) -> Result<Router, AppError> {
    let state = AppState::try_new(config).await?;

    let api_router = Router::new()
        .route("/chats", get(list_chat_handler).post(create_chat_handler))
        .route(
            "/chats/{id}",
            get(get_chat_handler)
                .patch(update_chat_handler)
                .delete(delete_chat_handler)
                .post(send_message_handler),
        )
        .route("/chats/{id}/messages", get(list_message_handler))
        .route("/messages/upload", post(upload_handler))
        .route("/files/{ws_id}/{*file_url}", get(download_handler))
        .route("/users/{ws_name}", get(user_list_handler))
        .layer(from_fn_with_state(state.clone(), verify_token))
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

    use super::*;

    impl AppState {
        #[allow(unused)]
        pub async fn new_for_test() -> Result<(TestPg, Self), AppError> {
            let config = AppConfig::load_for_test()?;

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

    pub async fn get_pg_and_pool(url: Option<&str>) -> (TestPg, PgPool) {
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
}
