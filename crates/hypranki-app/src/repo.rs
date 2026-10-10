use stores::db::pool::SqlitePool;
use stores::repository::deck::DeckRepository;

use crate::app::FromRef;

#[derive(Debug)]
pub struct RepoState {
    deck_repo: DeckRepository,
}
impl RepoState {
    pub fn new<I>(infra_state: &I) -> Self
    where
        SqlitePool: FromRef<I>,
    {
        let pool = SqlitePool::from_ref(infra_state);

        Self {
            deck_repo: DeckRepository::new(pool),
        }
    }
}
