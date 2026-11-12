use anyhow::Result;
use hypranki_storage::db::pool::SqlitePool;
use std::path::PathBuf;

use crate::app::FromRef;

use self::app_logger::AppLogger;
use self::app_path::AppPath;
pub mod app_logger;
pub mod app_path;

#[derive(Debug)]
pub struct InfraState {
    app_path: AppPath,
    sqlite_pool: SqlitePool,
    app_logger: AppLogger,
}

impl FromRef<InfraState> for AppPath {
    fn from_ref(input: &InfraState) -> Self {
        input.app_path.clone()
    }
}

impl FromRef<InfraState> for SqlitePool {
    fn from_ref(input: &InfraState) -> Self {
        input.sqlite_pool.clone()
    }
}

impl InfraState {
    pub async fn new(custom_project: Option<PathBuf>) -> Result<Self> {
        let app_path = AppPath::new(custom_project)?;
        let sqlite_pool = SqlitePool::new(app_path.db_path()).await?;
        let app_logger = AppLogger::new(app_path.cache_dir().join("log"), "hypr_anki.log", None);

        Ok(Self {
            app_path,
            sqlite_pool,
            app_logger,
        })
    }

    pub fn app_path(&self) -> &AppPath {
        &self.app_path
    }

    pub fn sqlite_pool(&self) -> &SqlitePool {
        &self.sqlite_pool
    }
    pub fn app_logger(&self) -> &AppLogger {
        &self.app_logger
    }
}

#[cfg(test)]
mod tests {

    use std::fs::{read_dir, read_to_string};

    use super::*;
    use tempfile::tempdir;
    use tracing::info;

    #[tokio::test]
    async fn test_infra_state_initialization() -> Result<()> {
        let temp = tempdir()?;
        let custom_project = temp.path().join("hypranki_project");

        assert!(!custom_project.exists());

        // 初始化整个 infra 模块
        let infra = InfraState::new(Some(custom_project.clone())).await?;

        // 验证自定义应用数据目录已成功创建
        assert!(custom_project.exists());
        assert!(infra.app_path().data_dir().exists());

        // 验证应用的配置目录与缓存目录已成功创建
        assert!(infra.app_path().config_dir().exists());
        assert!(infra.app_path().cache_dir().exists());

        // 验证 SQLite 数据库文件已成功生成在数据目录下
        let db_file = infra.app_path().db_path();
        assert_eq!(db_file, custom_project.join("app.db"));
        assert!(db_file.exists());

        // 验证通过 FromRef 能够正常提取子状态
        let extracted_path = AppPath::from_ref(&infra);
        let extracted_pool = SqlitePool::from_ref(&infra);
        assert_eq!(extracted_path.db_path(), db_file);
        assert!(!extracted_pool.inner().is_closed());

        // 验证日志文件夹已经创建
        let log_dir = infra.app_path().cache_dir().join("log");
        assert!(log_dir.is_dir());
        let message = format!("日志中有内容了! {}", custom_project.display());
        info!("{message}");
        // 释放拥有 WorkerGuard 的状态，等待后台日志线程刷新文件。
        drop(infra);

        let mut message_found = false;
        for entry in read_dir(log_dir)? {
            let path = entry?.path();
            // daily appender 会在文件名后追加日期。
            if path.is_file()
                && path.file_name().is_some_and(|name| {
                    name.to_string_lossy().starts_with("hypr_anki.log.")
                })
                && read_to_string(path)?.contains(&message)
            {
                message_found = true;
                break;
            }
        }
        assert!(message_found, "日志文件中未找到测试消息");

        Ok(())
    }
}
