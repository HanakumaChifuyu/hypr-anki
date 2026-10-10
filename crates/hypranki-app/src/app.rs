use std::path::PathBuf;

use anyhow::Result;

use crate::infra::InfraState;
use crate::repo::RepoState;

pub trait FromRef<T> {
    fn from_ref(input: &T) -> Self;
}

impl<T: Clone> FromRef<T> for T {
    fn from_ref(input: &T) -> Self {
        input.clone()
    }
}

#[derive(Debug)]
pub struct AppState {
    infra_state: InfraState,
    repo_state: RepoState,
}

impl AppState {
    pub async fn new(custom_project: Option<PathBuf>) -> Result<Self> {
        let infra_state = InfraState::new(custom_project).await?;
        let repo_state = RepoState::new(&infra_state);

        Ok(Self {
            infra_state,
            repo_state,
        })
    }
}
