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
            .idle_timeout(Duration::from_millis(10))
            .max_lifetime(Duration::from_millis(30))
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
}

