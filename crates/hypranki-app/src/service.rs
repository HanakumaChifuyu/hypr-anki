use std::sync::Arc;

use axum::extract::FromRef;
use hypranki_service::services::deck::DeckService;
use hypranki_storage::repository::deck::DeckRepository;

use crate::app;
use crate::repo::Repositories;

#[derive(Debug, FromRef, Clone)]
pub struct ServiceState {
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
            deck_service: Arc::new(deck_service),
        }
    }
}
