use anyhow::Result;
use sqlx::{Pool, Sqlite};

use crate::db::pool::SqlitePool;
pub use crate::models::note::{CreateNoteType, NoteType};

#[derive(Debug, Clone)]
pub struct NoteTypeRepository {
    pool: SqlitePool,
}

impl NoteTypeRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &Pool<Sqlite> {
        self.pool.inner()
    }

    pub async fn create(&self, input: &CreateNoteType) -> Result<NoteType> {
        Ok(sqlx::query_as::<_, NoteType>(
            "INSERT INTO NOTETYPES (name, fields) VALUES (?, ?) RETURNING id, name, fields",
        )
        .bind(&input.name)
        .bind(sqlx::types::Json(&input.fields))
        .fetch_one(self.pool())
        .await?)
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<NoteType>> {
        Ok(
            sqlx::query_as::<_, NoteType>("SELECT id, name, fields FROM NOTETYPES WHERE id = ?")
                .bind(id)
                .fetch_optional(self.pool())
                .await?,
        )
    }

    /// 按 ID 升序返回所有记录。
    pub async fn find_all(&self) -> Result<Vec<NoteType>> {
        Ok(
            sqlx::query_as::<_, NoteType>("SELECT id, name, fields FROM NOTETYPES ORDER BY id ASC")
                .fetch_all(self.pool())
                .await?,
        )
    }

    /// 全量更新；记录不存在时返回 false。
    pub async fn update(&self, input: &NoteType) -> Result<bool> {
        Self::update_on(&mut *self.pool().acquire().await?, input).await
    }

    pub async fn update_on(conn: &mut sqlx::SqliteConnection, input: &NoteType) -> Result<bool> {
        let result = sqlx::query("UPDATE NOTETYPES SET name = ?, fields = ? WHERE id = ?")
            .bind(&input.name)
            .bind(sqlx::types::Json(&input.fields))
            .bind(input.id)
            .execute(&mut *conn)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// 删除记录；记录不存在时返回 false。
    pub async fn delete_by_id(&self, id: i64) -> Result<bool> {
        Self::delete_by_id_on(&mut *self.pool().acquire().await?, id).await
    }

    pub async fn delete_by_id_on(conn: &mut sqlx::SqliteConnection, id: i64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM NOTETYPES WHERE id = ?")
            .bind(id)
            .execute(&mut *conn)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn find_by_id_on(
        conn: &mut sqlx::SqliteConnection,
        id: i64,
    ) -> Result<Option<NoteType>> {
        Ok(
            sqlx::query_as::<_, NoteType>("SELECT id, name, fields FROM NOTETYPES WHERE id = ?")
                .bind(id)
                .fetch_optional(&mut *conn)
                .await?,
        )
    }
}
