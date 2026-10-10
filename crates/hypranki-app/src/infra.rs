use anyhow::Result;
use std::path::PathBuf;
use stores::db::pool::SqlitePool;

use crate::app::FromRef;

use self::app_path::AppPath;
pub mod app_path;

#[derive(Debug)]
pub struct InfraState {
    app_path: AppPath,
    sqlite_pool: SqlitePool,
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

        Ok(Self {
            app_path,
            sqlite_pool,
        })
    }

    pub fn app_path(&self) -> &AppPath {
        &self.app_path
    }

    pub fn sqlite_pool(&self) -> &SqlitePool {
        &self.sqlite_pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_infra_state_initialization() -> Result<()> {
        let temp = tempdir()?;
        let custom_project = temp.path().join("hypranki_project");

        assert!(!custom_project.exists());

        // 初始化整个 infra 模块
        let infra = InfraState::new(Some(custom_project.clone())).await?;

        // 1. 验证自定义应用数据目录已成功创建
        assert!(custom_project.exists());
        assert!(infra.app_path().data_dir().exists());

        // 2. 验证应用的配置目录与缓存目录已成功创建
        assert!(infra.app_path().config_dir().exists());
        assert!(infra.app_path().cache_dir().exists());

        // 3. 验证 SQLite 数据库文件已成功生成在数据目录下
        let db_file = infra.app_path().db_path();
        assert_eq!(db_file, custom_project.join("app.db"));
        assert!(db_file.exists());

        // 4. 验证通过 FromRef 能够正常提取子状态
        let extracted_path = AppPath::from_ref(&infra);
        let extracted_pool = SqlitePool::from_ref(&infra);
        assert_eq!(extracted_path.db_path(), db_file);
        assert!(!extracted_pool.inner().is_closed());

        Ok(())
    }
}
