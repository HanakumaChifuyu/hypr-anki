use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use directories::ProjectDirs;

#[derive(Debug, Clone)]
pub struct AppPath {
    config_dir: PathBuf,
    cache_dir: PathBuf,
    data_dir: PathBuf,
}

impl AppPath {
    pub fn new(custom_project: Option<PathBuf>) -> Result<Self> {
        let project_dir = ProjectDirs::from("", "", "hypranki").expect("无法识别操作系统的主目录");

        let data_dir = custom_project.unwrap_or_else(|| project_dir.data_dir().to_path_buf());

        let app_path = Self {
            config_dir: project_dir.config_dir().to_path_buf(),
            cache_dir: project_dir.cache_dir().to_path_buf(),
            data_dir,
        };

        app_path.ensure_dirs()?;

        Ok(app_path)
    }

    /// 检查并创建所需的各个目录（配置目录、缓存目录、数据目录）
    pub fn ensure_dirs(&self) -> Result<()> {
        if !self.config_dir.exists() {
            fs::create_dir_all(&self.config_dir)?;
        }
        if !self.cache_dir.exists() {
            fs::create_dir_all(&self.cache_dir)?;
        }
        if !self.data_dir.exists() {
            fs::create_dir_all(&self.data_dir)?;
        }
        Ok(())
    }

    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn db_path(&self) -> PathBuf {
        self.data_dir.join("app.db")
    }
}

