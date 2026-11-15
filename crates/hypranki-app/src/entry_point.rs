use std::path::Path;

use anyhow::Result;

use crate::app::AppState;
use crate::infra::app_config::AppOptions;

pub async fn entry_point() -> Result<()> {
    // 开发配置固定在仓库中，避免受启动工作目录影响。
    let project_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let config_file = project_dir.join("config/development.toml");
    let data_dir = project_dir.join(".hypranki");
    let options = AppOptions::load_from_file(&config_file).unwrap_or_else(|error| {
        tracing::warn!(error = %format!("{error:#}"), "加载开发配置失败，使用默认配置");
        AppOptions::default()
    });
    let app_state = AppState::new(Some(&data_dir), Some(&config_file)).await?;
    app_state.run(options.listen_addr.to_string()).await
}
