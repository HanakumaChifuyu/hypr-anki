use crate::deck::DeckView;
use crate::path::AppPath;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::{fs, io};

use anyhow::{Ok, Result};

pub trait FromRef<T> {
    fn from_ref(input: &T) -> Self;
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub app_path: Arc<AppPath>,
    pub deck_view: Arc<DeckView>,
}

impl AppState {}

#[cfg(test)]
mod test {}
