use std::path::PathBuf;

use anyhow::Result;

use crate::app::AppState;

#[derive(Debug)]
pub struct LaunchOptions {
    // HYPRANKI_PROJECT_DIR 指定数据目录；未设置时使用系统默认目录。
    custom_project_dir: Option<PathBuf>,
}

impl LaunchOptions {
    pub fn from_envs() -> Self {
        let custom_project_dir = std::env::var_os("HYPRANKI_PROJECT_DIR")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from);

        Self { custom_project_dir }
    }
}

pub async fn entry_point() -> Result<()> {
    let options = LaunchOptions::from_envs();
    let app_state = AppState::new(options.custom_project_dir).await?;
    app_state.run("127.0.0.1:3000").await
}
