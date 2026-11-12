use axum::extract::{FromRef, State};
use axum::routing::get;
use axum::{Json, Router};
use hypranki_service::services::deck::DeckService;
use hypranki_storage::models::deck::Deck;

use crate::service::ServiceState;

use super::app_error::AppError;

pub struct DeckHandler;

impl DeckHandler {
    pub fn router() -> Router<ServiceState> {
        Router::new().route("/list", get(DeckHandler::list_deck))
    }

    async fn list_deck(
        State(ServiceState { deck_service, .. }): State<ServiceState>,
    ) -> Result<Json<Vec<Deck>>, AppError> {
        match deck_service.list().await {
            Ok(deck) => {
                return Ok(Json(deck));
            }

            Err(e) => {
                if let Some(db_err) = e.downcast_ref::<sqlx::Error>() {
                    return Err(AppError::InternalError(e));
                }
                return Err(AppError::BadRequest(e.to_string()));
            }
        }
    }
}
