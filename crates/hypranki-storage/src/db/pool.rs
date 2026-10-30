use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::{Pool, Sqlite};
#[derive(Debug, Clone)]
pub struct SqlitePool {
    pub inner_pool: Pool<Sqlite>,
}

impl SqlitePool {
    pub async fn init_pool(filename: impl AsRef<Path>) -> Result<Pool<Sqlite>> {
        let sqlite_option = SqliteConnectOptions::new()
            .filename(filename)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
            .busy_timeout(Duration::from_secs(5));

        Ok(SqlitePoolOptions::new()
            .max_connections(5)
            .min_connections(1)
            .acquire_timeout(Duration::from_secs(3))
            .idle_timeout(Duration::from_secs(10 * 60))
            .max_lifetime(Duration::from_secs(30 * 60))
            .connect_with(sqlite_option)
            .await?)
    }

    pub async fn new(filename: impl AsRef<Path>) -> Result<Self> {
        let inner_pool = Self::init_pool(filename).await?;
        Ok(Self { inner_pool })
    }

    pub fn inner(&self) -> &Pool<Sqlite> {
        &self.inner_pool
    }

    pub async fn migrate(&self, migr_dir: impl AsRef<Path>) -> Result<()> {
        let migrator = sqlx::migrate::Migrator::new(migr_dir.as_ref()).await?;
        migrator.run(self.inner()).await?;
        Ok(())
    }
}
