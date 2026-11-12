use anyhow::Result;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{Value, json};
use tokio::net::TcpListener;

use crate::service::ServiceState;

use self::deck_handler::DeckHandler;

pub mod app_error;
pub mod deck_handler;

pub struct AppHandler;

impl AppHandler {
    fn router(services: ServiceState) -> Router {
        let deck_router = DeckHandler::router();
        Router::new()
            .nest("/deck", deck_router)
            .with_state(services)
    }

    /// 启动所有handler
    pub async fn launcher(listenadd: impl AsRef<str>, services: ServiceState) -> Result<()> {
        let listener = TcpListener::bind(listenadd.as_ref()).await?;
        let routers = AppHandler::router(services);
        axum::serve(listener, routers).await?;

        Ok(())
    }
}
