use std::{fmt, ops::Deref, sync::Arc};

use axum::{
    Router,
    http::Method,
    middleware::from_fn_with_state,
    response::{Html, IntoResponse},
    routing::get,
};
use tower_http::cors::{self, CorsLayer};

mod config;
mod error;
mod notif;
mod sse;

use chat_core::{DecodingKey, TokenVerify, User, verify_token};
use dashmap::DashMap;

pub use config::*;
pub use error::*;
pub use notif::AppEvent;
pub use notif::*;
pub use sse::*;

use tokio::sync::broadcast;

pub type UserMap = Arc<DashMap<u64, broadcast::Sender<Arc<AppEvent>>>>;

#[derive(Debug, Clone)]
pub struct AppState(Arc<AppStateInner>);

pub struct AppStateInner {
    pub config: AppConfig,
    pub dk: DecodingKey,
    pub users: UserMap,
}

impl fmt::Debug for AppStateInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppStateInner")
            .field("config", &self.config)
            .finish()
    }
}

impl AppState {
    pub async fn new(config: AppConfig) -> Self {
        let dk = DecodingKey::load(&config.auth.pk).unwrap();
        let users = Arc::new(DashMap::new());
        Self(Arc::new(AppStateInner { config, dk, users }))
    }
}

pub async fn get_router(config: AppConfig) -> anyhow::Result<Router> {
    let state = AppState::new(config).await;
    setup_pg_listener(state.clone()).await?;

    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::PUT,
        ])
        .allow_origin(cors::Any)
        .allow_headers(cors::Any);

    let app = Router::new()
        .route("/events", get(sse::sse_handler))
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
        .layer(cors)
        .route("/", get(index_handler))
        .with_state(state.clone());

    Ok(app)
}

async fn index_handler() -> impl IntoResponse {
    let html = include_str!("../index.html");
    Html(html)
}

impl TokenVerify for AppState {
    type Error = AppError;

    fn verify(&self, token: &str) -> Result<User, Self::Error> {
        let user = self.dk.verify(token).map_err(AppError::from)?;
        Ok(user)
    }
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
