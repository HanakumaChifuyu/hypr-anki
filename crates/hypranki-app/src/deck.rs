use std::collections::HashSet;
use std::fs;
use std::path::{self, Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use directories::ProjectDirs;

use crate::app::{AppState, FromRef};
use crate::path::AppPath;

#[derive(Debug, Clone)]
pub struct Deck {
    pub id: u32,
    pub name: String,
}

pub struct DeckView {
    pub deck_tree: Vec<Deck>,
    pub collapsed_ids: Vec<u32>,
}

impl FromRef<AppState> for Arc<DeckView> {
    fn from_ref(input: &AppState) -> Self {
        Arc::clone(&input.deck_view)
    }
}

impl DeckView {
    pub fn new(deck_tree: Vec<Deck>, collapsed_ids: Vec<u32>) -> Self {
        Self {
            deck_tree,
            collapsed_ids,
        }
    }
}
