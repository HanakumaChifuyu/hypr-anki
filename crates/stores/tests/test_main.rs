use anyhow::Result;
use stores::db::pool;
use tempfile::tempdir;

#[tokio::test]
async fn test_sqlite_pool() -> Result<()> {
    let tempdir = tempdir()?;

    let pool = pool::SqlitePool::init_pool(tempdir.path().join("test.db")).await?;

    sqlx::query("CREATE TABLE IF NOT EXISTS test (id INTEGER PRIMARY KEY);")
        .execute(&pool)
        .await?;
    Ok(())
}
