use std::sync::Arc;

use axum::extract::FromRef;
use hypranki_service::services::deck::DeckService;
use hypranki_storage::repository::deck::DeckRepository;

use crate::app;
use crate::repo::Repositories;
use hypranki_service::services::card::CardService;
use hypranki_service::services::note_type::NoteTypeService;
use hypranki_storage::repository::note_type::NoteTypeRepository;
use hypranki_storage::{db::pool::SqlitePool, repository::card::CardRepository};

#[derive(Debug, FromRef, Clone)]
pub struct ServiceState {
    pub card_service: Arc<CardService>,
    pub deck_service: Arc<DeckService>,
    pub note_type_service: Arc<NoteTypeService>,
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
            note_type_service: Arc::new(NoteTypeService::new(Arc::new(
                <NoteTypeRepository as app::FromRef<_>>::from_ref(&repositories),
            ))),
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
            note_type_service: Arc::new(NoteTypeService::new(Arc::new(NoteTypeRepository::new(
                pool.clone(),
            )))),
            deck_service: Arc::new(DeckService::new(Arc::new(DeckRepository::new(
                pool.clone(),
            )))),
            card_service: Arc::new(CardService::new(Arc::new(CardRepository::new(pool)))),
        }
    }
}
