use hypranki_storage::db::pool::SqlitePool;
use hypranki_storage::repository::deck::DeckRepository;

use crate::app::FromRef;
use hypranki_storage::repository::card::CardRepository;

#[derive(Debug, Clone)]
pub struct Repositories {
    deck_repo: DeckRepository,
    card_repo: CardRepository,
}

impl FromRef<Repositories> for DeckRepository {
    fn from_ref(input: &Repositories) -> Self {
        input.deck_repo.clone()
    }
}

impl Repositories {
    pub fn new<I>(infra_state: &I) -> Self
    where
        SqlitePool: FromRef<I>,
    {
        let pool = SqlitePool::from_ref(infra_state);

        Self {
            deck_repo: DeckRepository::new(pool.clone()),
            card_repo: CardRepository::new(pool),
        }
    }
}

impl FromRef<Repositories> for CardRepository {
    fn from_ref(input: &Repositories) -> Self {
        input.card_repo.clone()
    }
}
