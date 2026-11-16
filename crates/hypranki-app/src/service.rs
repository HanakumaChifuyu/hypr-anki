use std::sync::Arc;

use axum::extract::FromRef;
use hypranki_service::services::deck::DeckService;
use hypranki_storage::repository::deck::DeckRepository;

use crate::app;
use crate::repo::Repositories;
use hypranki_service::services::card::CardService;
use hypranki_storage::{db::pool::SqlitePool, repository::card::CardRepository};

#[derive(Debug, FromRef, Clone)]
pub struct ServiceState {
    pub card_service: Arc<CardService>,
    pub deck_service: Arc<DeckService>,
}

impl ServiceState {
    pub fn new(repositories: Repositories) -> Self
    where
        DeckRepository: app::FromRef<Repositories>,
    {
        let deck_service = DeckService::new(Arc::new(
            <DeckRepository as app::FromRef<_>>::from_ref(&repositories),
        ));

        Self {
            card_service: Arc::new(CardService::new(Arc::new(
                <CardRepository as app::FromRef<_>>::from_ref(&repositories),
            ))),
            deck_service: Arc::new(deck_service),
        }
    }
}

impl ServiceState {
    pub fn from_pool(pool: SqlitePool) -> Self {
        Self {
            deck_service: Arc::new(DeckService::new(Arc::new(DeckRepository::new(
                pool.clone(),
            )))),
            card_service: Arc::new(CardService::new(Arc::new(CardRepository::new(pool)))),
        }
    }
}
