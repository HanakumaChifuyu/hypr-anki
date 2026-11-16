use crate::{db::pool::SqlitePool, models::template::Template};
use anyhow::Result;
use sqlx::{Pool, Sqlite, SqliteConnection};

#[derive(Debug, Clone)]
pub struct TemplateRepository {
    pool: SqlitePool,
}
impl TemplateRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    pub fn pool(&self) -> &Pool<Sqlite> {
        self.pool.inner()
    }
    pub async fn find_by_id(&self, id: i64) -> Result<Option<Template>> {
        Self::find_by_id_on(&mut *self.pool().acquire().await?, id).await
    }
    pub async fn find_by_id_on(conn: &mut SqliteConnection, id: i64) -> Result<Option<Template>> {
        Ok(
            sqlx::query_as("SELECT id, name FROM TEMPLATES WHERE id = ?")
                .bind(id)
                .fetch_optional(conn)
                .await?,
        )
    }
}
