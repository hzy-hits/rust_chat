mod config;
mod error;
mod handlers;
mod models;
mod utils;

use anyhow::Context;
use axum::{
    routing::{get, patch, post},
    Router,
};
pub use config::AppConfig;
pub use error::AppError;
use handlers::*;
pub use models::User;
pub use utils::jwt::{DecodingKey, EncodingKey};

use core::fmt;
use std::{ops::Deref, sync::Arc};
#[derive(Debug, Clone)]
pub(crate) struct AppState {
    pub inner: Arc<AppStateInner>,
}
#[allow(unused)]
pub(crate) struct AppStateInner {
    pub(crate) config: AppConfig,
    pub(crate) dk: DecodingKey,
    pub(crate) ek: EncodingKey,
    pub(crate) pg_pool: sqlx::PgPool,
}
pub async fn get_router(config: AppConfig) -> Result<Router, AppError> {
    let state = AppState::try_new(config).await?;
    let api = Router::new()
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler))
        .route("/chat", get(list_char_handler).post(create_chat_handler))
        .route(
            "/chat/:id",
            patch(update_chat_handler).delete(delete_chat_handler),
        )
        .route("/chat/:id/messages", get(list_messages_handler));
    let app = Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state);
    Ok(app)
}

impl Deref for AppState {
    type Target = AppStateInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
        let dk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
        let ek = EncodingKey::load(&config.auth.sk).context("load sk failed")?;
        let pool = sqlx::PgPool::connect(&config.server.db_url)
            .await
            .context("connect to db failed")?;
        Ok(Self {
            inner: Arc::new(AppStateInner {
                config,
                dk,
                ek,
                pg_pool: pool,
            }),
        })
    }
}

impl fmt::Debug for AppStateInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppStateInner")
            .field("config", &self.config)
            .finish()
    }
}
