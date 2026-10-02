use crate::app::FromRef;
use anyhow::{Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use directories::ProjectDirs;

#[derive(Debug, Clone)]
pub struct AppPath {
    pub project_dirs: PathBuf,
    pub config_dir: PathBuf,
    pub cache_dir: PathBuf,
}

impl FromRef<AppState> for Arc<AppPath> {
    fn from_ref(input: &AppState) -> Self {
        Arc::clone(&input.app_path)
    }
}

impl AppPath {
    pub const DOMAIN: &'static str = "icu";
    pub const ORGANIZATION: &'static str = "Dingbird";
    pub const APPNAME: &'static str = "hypranki";
    fn default_project_dir() -> Result<ProjectDirs> {
        Ok(
            ProjectDirs::from(AppPath::DOMAIN, AppPath::ORGANIZATION, AppPath::APPNAME)
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        "Unable to locate system standard directory",
                    )
                })?,
        )
    }

    pub fn init_app_dir() -> Result<AppPath> {
        let project_dirs = Self::default_project_dir()?;

        let config_dir = project_dirs.config_dir();
        let cache_dir = project_dirs.cache_dir();

        if !cache_dir.exists() {
            fs::create_dir(cache_dir)?;
        }
        if !config_dir.exists() {
            fs::create_dir(config_dir)?;
        }

        Ok(AppPath {
            project_dirs: project_dirs.project_path().to_path_buf(),
            config_dir: config_dir.to_path_buf(),
            cache_dir: cache_dir.to_path_buf(),
        })
    }

    pub fn init_app_dir_with_custom(custom: impl AsRef<Path>) -> Result<Self> {
        let custom = custom.as_ref();
        let project_name = format!(
            "{}.{}.{}",
            AppPath::DOMAIN,
            AppPath::ORGANIZATION,
            AppPath::APPNAME
        );

        let project_dirs = custom.join(project_name);
        let config_dir = project_dirs.join("config");
        let cache_dir = project_dirs.join("cache");

        Ok(Self {
            project_dirs,
            config_dir,
            cache_dir,
        })
    }
    pub fn cache_path(&self) -> Result<&Path> {
        let cache_path = self.cache_dir.as_path();
        match fs::metadata(cache_path) {
            Ok(meta) => Ok(cache_path),
            Err(e) => Err(e.into()),
        }
    }
}
