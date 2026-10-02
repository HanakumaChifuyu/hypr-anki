use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::path::AppPath;
use anyhow::Result;
use serde::Serialize;
use tempfile::NamedTempFile;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct DeckTreeState {
    pub collapsed_ids: Vec<u32>,
}
impl DeckTreeState {
    pub fn load(&self, app_path: AppPath) {}

    pub fn save(&self, app_path: AppPath) -> Result<()> {
        let cache_path = app_path.cache_path()?;
        let deck_tree_path = cache_path.join("deck-tree.json");
        let temp_file = NamedTempFile::new_in(cache_path)?;
        {
            let mut temp_writer = BufWriter::new(&temp_file);
            serde_json::to_writer_pretty(temp_writer, &self.collapsed_ids)?;
            temp_writer.flush()?;
            temp_file.into_file().sync_all();
        }

        temp_file.persist(deck_tree_path)?;
        Ok(())
    }
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_decktree_save() {
        let temp_dir = tempfile::tempdir().unwrap();
        let app_path = AppPath::init_app_dir_with_custom(temp_dir).unwrap();
        let deck_tree_state = DeckTreeState {
            collapsed_ids: vec![1, 2, 3, 4, 5],
        };
        deck_tree_state.save(app_path).unwrap();
    }
}
