use std::path::PathBuf;

use anyhow::Result;

use crate::app::AppState;

#[derive(Debug)]
pub struct LaunchOptions {
    // 项目的指定路径
    custom_project_dir: Option<PathBuf>,
}

impl LaunchOptions {
    pub fn from_envs() -> Self {
        todo!()
    }
}

pub async fn entry_point() -> Result<()> {
    let app_state = AppState::new(None).await?;
    Ok(())
}
