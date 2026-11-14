use axum::extract::{FromRef, State};
use axum::routing::get;
use axum::{Json, Router};
use hypranki_service::services::deck::DeckService;
use hypranki_storage::models::deck::Deck;
use tracing::{info, warn};

use crate::service::ServiceState;

use super::app_error::AppError;

pub struct DeckHandler;

impl DeckHandler {
    pub fn router() -> Router<ServiceState> {
        Router::new().route("/list", get(DeckHandler::list_deck))
    }

    #[tracing::instrument(skip_all, fields(method = "GET", path = "/deck/list"))]
    async fn list_deck(
        State(ServiceState { deck_service, .. }): State<ServiceState>,
    ) -> Result<Json<Vec<Deck>>, AppError> {
        info!("开始查询牌组列表");
        match deck_service.list().await {
            Ok(deck) => {
                info!(status = 200, count = deck.len(), "查询牌组列表成功");
                return Ok(Json(deck));
            }

            Err(e) => {
                if e.downcast_ref::<sqlx::Error>().is_some() {
                    // AppError 会统一打印数据库错误详情。
                    warn!(status = 500, "查询牌组列表失败：数据库错误");
                    return Err(AppError::InternalError(e));
                }
                warn!(status = 400, error = %e, "查询牌组列表失败");
                return Err(AppError::BadRequest(e.to_string()));
            }
        }
    }
}
