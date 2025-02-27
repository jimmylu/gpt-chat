mod config;
mod error;
mod handlers;
mod models;
mod utils;

use core::fmt;
use std::{ops::Deref, sync::Arc};

use anyhow::Context;
use axum::{
    routing::{get, post},
    Router,
};
use handlers::*;
use sqlx::PgPool;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;
use utils::{DecodingKey, EncodingKey};

pub use config::AppConfig;
pub use error::AppError;
pub use models::User;
pub async fn get_router(config: AppConfig) -> Result<Router, AppError> {
    let state = AppState::try_new(config).await?;

    let api_router = Router::new()
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler))
        .route("/chat", get(list_chat_handler).post(create_chat_handler))
        .route(
            "/chat/{id}",
            get(list_chat_handler)
                .patch(update_chat_handler)
                .delete(delete_chat_handler),
        )
        .layer(
            ServiceBuilder::new()
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().include_headers(true))
                        .on_request(DefaultOnRequest::new().level(Level::INFO))
                        .on_response(
                            DefaultOnResponse::new()
                                .level(Level::INFO)
                                .latency_unit(LatencyUnit::Micros),
                        ),
                )
                .layer(CompressionLayer::new().gzip(true).br(true).deflate(true)),
        );

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
    pub sk: EncodingKey,
    pub pk: DecodingKey,
    pub pool: PgPool,
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
impl AppState {
    #[allow(unused)]
    pub async fn new_for_test() -> Result<(sqlx_db_tester::TestPg, Self), AppError> {
        let config = AppConfig::load_for_test()?;

        let sk = EncodingKey::load(&config.auth.sk).context("load sk failed")?;
        let pk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
        let url = config.server.db_url.rfind('/').expect("db_url invalid");
        let tdb = sqlx_db_tester::TestPg::new(
            config.server.db_url[..url].to_string(),
            std::path::Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
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
