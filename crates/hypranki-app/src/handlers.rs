use anyhow::Result;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{Value, json};
use tokio::net::TcpListener;

use crate::service::ServiceState;

use self::deck_handler::{ApiDoc, DeckHandler};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

pub mod app_error;
pub mod deck_handler;

pub struct AppHandler;

impl AppHandler {
    fn router(services: ServiceState) -> Router {
        let deck_router = DeckHandler::router();
        Router::new()
            .nest("/deck", deck_router)
            .with_state(services)
            .route(
                "/api-docs/openapi.json",
                get(|| async { Json(ApiDoc::openapi()) }),
            )
            .merge(Scalar::with_url("/scalar", ApiDoc::openapi()))
    }

    /// 启动所有handler
    pub async fn launcher(listenadd: impl AsRef<str>, services: ServiceState) -> Result<()> {
        let listener = TcpListener::bind(listenadd.as_ref()).await?;
        let routers = AppHandler::router(services);
        axum::serve(listener, routers).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use hypranki_service::services::deck::DeckService;
    use hypranki_storage::{db::pool::SqlitePool, repository::deck::DeckRepository};
    use std::sync::Arc;
    use tower::ServiceExt;

    #[tokio::test]
    async fn serves_scalar_and_complete_openapi() {
        let dir = tempfile::tempdir().unwrap();
        let pool = SqlitePool::new(dir.path().join("docs.db")).await.unwrap();
        let router = AppHandler::router(ServiceState::from_pool(pool));
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/scalar")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            response.headers()["content-type"]
                .to_str()
                .unwrap()
                .contains("text/html")
        );
        let html = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        assert!(String::from_utf8(html.to_vec()).unwrap().contains("scalar"));
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/api-docs/openapi.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        let doc: Value = serde_json::from_slice(&bytes).unwrap();
        for (path, method) in [
            ("/deck/list", "get"),
            ("/deck", "post"),
            ("/deck/{id}", "get"),
            ("/deck/{id}", "patch"),
            ("/deck/{id}", "delete"),
            ("/deck/{id}/rename", "patch"),
            ("/deck/{id}/move", "patch"),
        ] {
            assert!(doc["paths"][path][method].is_object(), "{method} {path}");
            assert!(doc["paths"][path][method]["responses"]["500"].is_object());
        }
        let schemas = &doc["components"]["schemas"];
        assert!(
            schemas["MoveDeck"]["required"]
                .as_array()
                .unwrap()
                .contains(&json!("parent_id"))
        );
        assert!(
            !schemas["UpdateDeck"]["required"]
                .as_array()
                .is_some_and(|fields| fields.contains(&json!("parent_id")))
        );
    }
}
