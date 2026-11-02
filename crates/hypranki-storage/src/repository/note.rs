use anyhow::Result;
use sqlx::{Pool, Sqlite};

use crate::db::pool::SqlitePool;
pub use crate::models::note::{CreateNote, Note};

#[derive(Debug, Clone)]
pub struct NoteRepository {
    pool: SqlitePool,
}

impl NoteRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &Pool<Sqlite> {
        self.pool.inner()
    }

    pub async fn create(&self, input: &CreateNote) -> Result<Note> {
        Ok(sqlx::query_as::<_, Note>(
            "INSERT INTO NOTES (note_type, content) VALUES (?, ?) RETURNING id, note_type, content",
        )
        .bind(input.note_type)
        .bind(sqlx::types::Json(&input.content))
        .fetch_one(self.pool())
        .await?)
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<Note>> {
        Ok(
            sqlx::query_as::<_, Note>("SELECT id, note_type, content FROM NOTES WHERE id = ?")
                .bind(id)
                .fetch_optional(self.pool())
                .await?,
        )
    }

    /// 按 ID 升序返回所有记录。
    pub async fn find_all(&self) -> Result<Vec<Note>> {
        Ok(
            sqlx::query_as::<_, Note>("SELECT id, note_type, content FROM NOTES ORDER BY id ASC")
                .fetch_all(self.pool())
                .await?,
        )
    }

    /// 全量更新；记录不存在时返回 false。
    pub async fn update(&self, input: &Note) -> Result<bool> {
        let result = sqlx::query("UPDATE NOTES SET note_type = ?, content = ? WHERE id = ?")
            .bind(input.note_type)
            .bind(sqlx::types::Json(&input.content))
            .bind(input.id)
            .execute(self.pool())
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// 删除记录；记录不存在时返回 false。
    pub async fn delete_by_id(&self, id: i64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM NOTES WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn find_by_note_type(&self, note_type: i64) -> Result<Vec<Note>> {
        Ok(sqlx::query_as::<_, Note>(
            "SELECT id, note_type, content FROM NOTES WHERE note_type = ? ORDER BY id ASC",
        )
        .bind(note_type)
        .fetch_all(self.pool())
        .await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::note::{CreateNoteType, Field, FieldType};
    use crate::repository::note_type::NoteTypeRepository;
    use serde_json::json;

    #[tokio::test]
    async fn note_and_note_type_crud() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let pool = SqlitePool::new(directory.path().join("notes.db")).await?;
        sqlx::migrate!("../../migrations").run(pool.inner()).await?;
        let types = NoteTypeRepository::new(pool.clone());
        let notes = NoteRepository::new(pool);
        assert!(types.find_all().await?.is_empty());
        assert!(notes.find_all().await?.is_empty());

        let mut note_type = types
            .create(&CreateNoteType {
                name: "Basic".into(),
                fields: vec![Field {
                    name: "Front".into(),
                    field_type: FieldType::Str,
                    writability: true,
                }],
            })
            .await?;
        let loaded = types.find_by_id(note_type.id).await?.unwrap();
        assert_eq!(loaded.name, "Basic");
        assert_eq!(loaded.fields[0].name, "Front");
        assert!(loaded.fields[0].writability);
        assert!(matches!(loaded.fields[0].field_type, FieldType::Str));
        note_type.name = "Renamed".into();
        note_type.fields[0].writability = false;
        note_type.fields.push(Field {
            name: "Extra".into(),
            field_type: FieldType::Json,
            writability: false,
        });
        assert!(types.update(&note_type).await?);
        let loaded = types.find_all().await?;
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "Renamed");
        assert_eq!(loaded[0].fields.len(), 2);
        assert!(!loaded[0].fields[0].writability);
        assert!(!loaded[0].fields[1].writability);

        let mut note = notes
            .create(&CreateNote {
                note_type: note_type.id,
                content: json!({"Front": "hello", "Extra": {"nested": [1, true]}}),
            })
            .await?;
        assert_eq!(
            notes.find_by_id(note.id).await?.unwrap().content,
            note.content
        );
        assert_eq!(notes.find_all().await?.len(), 1);
        assert_eq!(notes.find_by_note_type(note_type.id).await?.len(), 1);
        assert!(notes.find_by_note_type(-1).await?.is_empty());
        note.content = json!({"Front": "updated", "Extra": {}});
        assert!(notes.update(&note).await?);
        assert_eq!(
            notes.find_by_id(note.id).await?.unwrap().content,
            note.content
        );

        // 外键约束应拒绝孤立笔记及删除仍被引用的笔记类型。
        assert!(
            notes
                .create(&CreateNote {
                    note_type: -1,
                    content: json!({})
                })
                .await
                .is_err()
        );
        assert!(types.delete_by_id(note_type.id).await.is_err());
        assert!(notes.delete_by_id(note.id).await?);
        assert!(!notes.delete_by_id(note.id).await?);
        assert!(notes.find_by_id(note.id).await?.is_none());
        assert!(!notes.update(&note).await?);
        assert!(types.delete_by_id(note_type.id).await?);
        assert!(!types.delete_by_id(note_type.id).await?);
        assert!(types.find_by_id(note_type.id).await?.is_none());
        assert!(!types.update(&note_type).await?);
        Ok(())
    }
}
