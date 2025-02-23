mod config;
mod handlers;

use std::{ops::Deref, sync::Arc};

use axum::{
    routing::{get, post},
    Router,
};
pub use config::AppConfig;
use handlers::*;

pub fn get_router(config: AppConfig) -> Router {
    let state = AppState::new(config);

    let api_router = Router::new()
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler))
        .route("/chat", get(list_chat_handler).post(create_chat_handler))
        .route(
            "/chat/{id}",
            get(list_chat_handler)
                .patch(update_chat_handler)
                .delete(delete_chat_handler),
        );

    Router::new()
        .route("/", get(index))
        .nest("/api", api_router)
        .with_state(state)
}

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    inner: Arc<AppStateInner>,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            inner: Arc::new(AppStateInner { config }),
        }
    }
}

#[allow(unused)]
#[derive(Debug)]
struct AppStateInner {
    config: AppConfig,
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
