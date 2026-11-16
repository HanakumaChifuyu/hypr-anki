use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use sqlx::{Pool, Sqlite};

use crate::db::pool::SqlitePool;
pub use crate::models::card::{Card, CreateCard, StoredMemoryState};

fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[derive(Debug, Clone)]
pub struct CardRepository {
    pool: SqlitePool,
}

impl CardRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &Pool<Sqlite> {
        self.pool.inner()
    }

    /// 同一快照读取当前页与筛选后的总数，避免并发写入造成二者不一致。
    pub async fn find_page(
        &self,
        deck_id: Option<i64>,
        limit: u32,
        offset: i64,
    ) -> Result<(Vec<Card>, i64)> {
        anyhow::ensure!((1..=200).contains(&limit), "每页数量必须在 1 到 200 之间");
        anyhow::ensure!(offset >= 0, "分页偏移量不能为负数");
        let mut tx = self.pool().begin().await?;
        let (items, total) = if let Some(deck_id) = deck_id {
            let total = sqlx::query_scalar("SELECT COUNT(*) FROM CARDS WHERE deck_id = ?")
                .bind(deck_id)
                .fetch_one(&mut *tx)
                .await?;
            let items = sqlx::query_as(
                "SELECT * FROM CARDS WHERE deck_id = ? ORDER BY id ASC LIMIT ? OFFSET ?",
            )
            .bind(deck_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&mut *tx)
            .await?;
            (items, total)
        } else {
            let total = sqlx::query_scalar("SELECT COUNT(*) FROM CARDS")
                .fetch_one(&mut *tx)
                .await?;
            let items = sqlx::query_as("SELECT * FROM CARDS ORDER BY id ASC LIMIT ? OFFSET ?")
                .bind(limit)
                .bind(offset)
                .fetch_all(&mut *tx)
                .await?;
            (items, total)
        };
        tx.commit().await?;
        Ok((items, total))
    }

    /// 新卡没有记忆状态；关联记录必须存在，且 Note 不能被其他卡片占用。
    pub async fn create(&self, input: &CreateCard) -> Result<Card> {
        let now = current_timestamp();
        Ok(sqlx::query_as::<_, Card>(
            "INSERT INTO CARDS (note_id, template_id, deck_id, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?) RETURNING *",
        )
        .bind(input.note_id)
        .bind(input.template_id)
        .bind(input.deck_id)
        .bind(now)
        .bind(now)
        .fetch_one(self.pool())
        .await?)
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<Card>> {
        Ok(
            sqlx::query_as::<_, Card>("SELECT * FROM CARDS WHERE id = ?")
                .bind(id)
                .fetch_optional(self.pool())
                .await?,
        )
    }

    pub async fn find_by_note_id(&self, note_id: i64) -> Result<Option<Card>> {
        Ok(
            sqlx::query_as::<_, Card>("SELECT * FROM CARDS WHERE note_id = ?")
                .bind(note_id)
                .fetch_optional(self.pool())
                .await?,
        )
    }

    /// 列表查询均按 ID 升序排列。
    pub async fn find_all(&self) -> Result<Vec<Card>> {
        Ok(
            sqlx::query_as::<_, Card>("SELECT * FROM CARDS ORDER BY id ASC")
                .fetch_all(self.pool())
                .await?,
        )
    }

    pub async fn find_by_deck_id(&self, deck_id: i64) -> Result<Vec<Card>> {
        Ok(
            sqlx::query_as::<_, Card>("SELECT * FROM CARDS WHERE deck_id = ? ORDER BY id ASC")
                .bind(deck_id)
                .fetch_all(self.pool())
                .await?,
        )
    }

    pub async fn find_by_template_id(&self, template_id: i64) -> Result<Vec<Card>> {
        Ok(
            sqlx::query_as::<_, Card>("SELECT * FROM CARDS WHERE template_id = ? ORDER BY id ASC")
                .bind(template_id)
                .fetch_all(self.pool())
                .await?,
        )
    }

    pub async fn exists_by_id(&self, id: i64) -> Result<bool> {
        Ok(
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM CARDS WHERE id = ?)")
                .bind(id)
                .fetch_one(self.pool())
                .await?,
        )
    }

    pub async fn count(&self) -> Result<i64> {
        Ok(sqlx::query_scalar("SELECT COUNT(*) FROM CARDS")
            .fetch_one(self.pool())
            .await?)
    }

    /// 全量更新可变字段，保留 created_at，自动更新 updated_at。
    /// 记录不存在时返回 false；None 记忆状态存为 SQL NULL。
    pub async fn update(&self, input: &Card) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE CARDS SET note_id = ?, template_id = ?, deck_id = ?, memory_state = ?, updated_at = ? WHERE id = ?",
        )
        .bind(input.note_id)
        .bind(input.template_id)
        .bind(input.deck_id)
        .bind(input.memory_state.map(sqlx::types::Json))
        .bind(current_timestamp())
        .bind(input.id)
        .execute(self.pool())
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn update_deck(&self, id: i64, deck_id: i64) -> Result<bool> {
        let result = sqlx::query("UPDATE CARDS SET deck_id = ?, updated_at = ? WHERE id = ?")
            .bind(deck_id)
            .bind(current_timestamp())
            .bind(id)
            .execute(self.pool())
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn update_template(&self, id: i64, template_id: i64) -> Result<bool> {
        let result = sqlx::query("UPDATE CARDS SET template_id = ?, updated_at = ? WHERE id = ?")
            .bind(template_id)
            .bind(current_timestamp())
            .bind(id)
            .execute(self.pool())
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// 更新记忆状态；None 表示清空，其他字段保持不变。
    pub async fn update_memory_state(
        &self,
        id: i64,
        state: Option<&StoredMemoryState>,
    ) -> Result<bool> {
        let result = sqlx::query("UPDATE CARDS SET memory_state = ?, updated_at = ? WHERE id = ?")
            .bind(state.map(sqlx::types::Json))
            .bind(current_timestamp())
            .bind(id)
            .execute(self.pool())
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// 只删除卡片，保留其内容和模板；关联资源生命周期由 service 管理。
    pub async fn delete_by_id(&self, id: i64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM CARDS WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn create_on(conn: &mut sqlx::SqliteConnection, input: &CreateCard) -> Result<Card> {
        let now = current_timestamp();
        Ok(sqlx::query_as::<_, Card>(
            "INSERT INTO CARDS (note_id, template_id, deck_id, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?) RETURNING *",
        )
        .bind(input.note_id)
        .bind(input.template_id)
        .bind(input.deck_id)
        .bind(now)
        .bind(now)
        .fetch_one(&mut *conn)
        .await?)
    }

    pub async fn find_by_id_on(conn: &mut sqlx::SqliteConnection, id: i64) -> Result<Option<Card>> {
        Ok(
            sqlx::query_as::<_, Card>("SELECT * FROM CARDS WHERE id = ?")
                .bind(id)
                .fetch_optional(&mut *conn)
                .await?,
        )
    }

    pub async fn update_on(conn: &mut sqlx::SqliteConnection, input: &Card) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE CARDS SET note_id = ?, template_id = ?, deck_id = ?, memory_state = ?, updated_at = ? WHERE id = ?",
        )
        .bind(input.note_id)
        .bind(input.template_id)
        .bind(input.deck_id)
        .bind(input.memory_state.map(sqlx::types::Json))
        .bind(current_timestamp())
        .bind(input.id)
        .execute(&mut *conn)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn update_deck_on(
        conn: &mut sqlx::SqliteConnection,
        id: i64,
        deck_id: i64,
    ) -> Result<bool> {
        let result = sqlx::query("UPDATE CARDS SET deck_id = ?, updated_at = ? WHERE id = ?")
            .bind(deck_id)
            .bind(current_timestamp())
            .bind(id)
            .execute(&mut *conn)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn update_template_on(
        conn: &mut sqlx::SqliteConnection,
        id: i64,
        template_id: i64,
    ) -> Result<bool> {
        let result = sqlx::query("UPDATE CARDS SET template_id = ?, updated_at = ? WHERE id = ?")
            .bind(template_id)
            .bind(current_timestamp())
            .bind(id)
            .execute(&mut *conn)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_by_id_on(conn: &mut sqlx::SqliteConnection, id: i64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM CARDS WHERE id = ?")
            .bind(id)
            .execute(&mut *conn)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> Result<(CardRepository, tempfile::TempDir)> {
        let dir = tempfile::tempdir()?;
        let pool = SqlitePool::new(dir.path().join("cards.db")).await?;
        pool.initialize_database().await?;
        sqlx::raw_sql(
            "INSERT INTO DECKS (id, name, config_id, created_at, updated_at) VALUES (1, 'A', 1, 0, 0), (2, 'B', 1, 0, 0);
             INSERT INTO NOTETYPES (id, name, fields) VALUES (1, 'Basic', '[]');
             INSERT INTO NOTES (id, note_type, content) VALUES (1, 1, '{}'), (2, 1, '{}'), (3, 1, '{}');
             INSERT INTO TEMPLATES (id, name) VALUES (1, 'A'), (2, 'B');",
        ).execute(pool.inner()).await?;
        Ok((CardRepository::new(pool), dir))
    }

    #[tokio::test]
    async fn card_crud_filters_and_memory_state() -> Result<()> {
        let (repo, _dir) = setup().await?;
        assert_eq!(repo.count().await?, 0);
        assert!(repo.find_all().await?.is_empty());
        let mut a = repo
            .create(&CreateCard {
                note_id: 1,
                template_id: 1,
                deck_id: 1,
            })
            .await?;
        let b = repo
            .create(&CreateCard {
                note_id: 2,
                template_id: 1,
                deck_id: 1,
            })
            .await?;
        assert!(a.memory_state.is_none());
        assert_eq!(a.created_at, a.updated_at);
        assert!(repo.exists_by_id(a.id).await?);
        assert_eq!(repo.find_by_id(a.id).await?, Some(a.clone()));
        assert_eq!(repo.find_by_note_id(1).await?, Some(a.clone()));
        assert_eq!(repo.find_all().await?, vec![a.clone(), b.clone()]);
        assert_eq!(repo.find_by_deck_id(1).await?, vec![a.clone(), b.clone()]);
        assert_eq!(
            repo.find_by_template_id(1).await?,
            vec![a.clone(), b.clone()]
        );

        let memory = StoredMemoryState {
            stability: 3.0,
            stability_fast: 2.4,
            difficulty: 5.0,
        };
        assert!(repo.update_memory_state(a.id, Some(&memory)).await?);
        let loaded = repo.find_by_id(a.id).await?.unwrap();
        assert_eq!(loaded.memory_state, Some(memory));
        assert_eq!(
            (
                loaded.note_id,
                loaded.deck_id,
                loaded.template_id,
                loaded.created_at
            ),
            (a.note_id, a.deck_id, a.template_id, a.created_at)
        );
        assert!(repo.update_memory_state(a.id, None).await?);
        assert!(repo.find_by_id(a.id).await?.unwrap().memory_state.is_none());

        assert!(repo.update_deck(a.id, 2).await?);
        assert!(repo.update_template(a.id, 2).await?);
        a = repo.find_by_id(a.id).await?.unwrap();
        assert_eq!((a.deck_id, a.template_id), (2, 2));
        assert_eq!(repo.find_by_deck_id(1).await?, vec![b.clone()]);
        assert_eq!(repo.find_by_template_id(1).await?, vec![b]);

        let created_at = a.created_at;
        a.note_id = 3;
        a.memory_state = Some(memory);
        a.created_at = 0;
        assert!(repo.update(&a).await?);
        let loaded = repo.find_by_id(a.id).await?.unwrap();
        assert_eq!(loaded.created_at, created_at);
        assert_eq!(loaded.note_id, 3);
        assert_eq!(loaded.memory_state, Some(memory));
        a.memory_state = None;
        assert!(repo.update(&a).await?);
        assert!(repo.find_by_id(a.id).await?.unwrap().memory_state.is_none());
        assert!(repo.delete_by_id(a.id).await?);
        assert!(!repo.delete_by_id(a.id).await?);
        let remaining_notes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM NOTES")
            .fetch_one(repo.pool())
            .await?;
        assert_eq!(remaining_notes, 3);
        assert_eq!(repo.count().await?, 1);
        Ok(())
    }

    #[tokio::test]
    async fn missing_records_and_invalid_relations() -> Result<()> {
        let (repo, _dir) = setup().await?;
        assert!(repo.find_by_id(999).await?.is_none());
        assert!(repo.find_by_note_id(999).await?.is_none());
        assert!(repo.find_by_deck_id(999).await?.is_empty());
        assert!(repo.find_by_template_id(999).await?.is_empty());
        assert!(!repo.exists_by_id(999).await?);
        assert!(!repo.update_deck(999, 1).await?);
        assert!(!repo.update_template(999, 1).await?);
        assert!(!repo.update_memory_state(999, None).await?);
        assert!(!repo.delete_by_id(999).await?);
        let a = repo
            .create(&CreateCard {
                note_id: 1,
                template_id: 1,
                deck_id: 1,
            })
            .await?;
        let mut missing = a.clone();
        missing.id = 999;
        assert!(!repo.update(&missing).await?);
        for (note_id, template_id, deck_id) in [(1, 1, 1), (99, 1, 1), (2, 99, 1), (2, 1, 99)] {
            assert!(
                repo.create(&CreateCard {
                    note_id,
                    template_id,
                    deck_id
                })
                .await
                .is_err()
            );
        }
        assert!(repo.update_deck(a.id, 999).await.is_err());
        assert!(repo.update_template(a.id, 999).await.is_err());
        let b = repo
            .create(&CreateCard {
                note_id: 2,
                template_id: 1,
                deck_id: 1,
            })
            .await?;
        let mut invalid = a.clone();
        invalid.note_id = b.note_id;
        assert!(repo.update(&invalid).await.is_err());
        assert_eq!(repo.find_by_id(a.id).await?, Some(a));
        assert_eq!(repo.count().await?, 2);
        Ok(())
    }
}

/// 复习历史保留算法版本、权重和前后状态，供重算与训练使用。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema, sqlx::FromRow)]
pub struct CardReview {
    pub id: i64,
    pub card_id: i64,
    pub reviewed_at: i64,
    pub rating: i64,
    pub elapsed_seconds: i64,
    pub scheduled_seconds: i64,
    #[sqlx(json(nullable))]
    pub memory_before: Option<StoredMemoryState>,
    #[sqlx(json)]
    pub memory_after: StoredMemoryState,
    pub algorithm: String,
    #[sqlx(json)]
    pub parameters: Vec<f32>,
    pub desired_retention: f32,
}

impl CardRepository {
    /// 仅返回已经安排复习且到期的卡片，新卡不在此队列。
    pub async fn find_due(&self, timestamp: i64, limit: u32) -> Result<Vec<Card>> {
        Ok(sqlx::query_as(
            "SELECT * FROM CARDS WHERE due_at <= ? ORDER BY due_at ASC, id ASC LIMIT ?",
        )
        .bind(timestamp)
        .bind(limit)
        .fetch_all(self.pool())
        .await?)
    }

    pub async fn save_review_on(
        conn: &mut sqlx::SqliteConnection,
        review: &CardReview,
    ) -> Result<Card> {
        sqlx::query("INSERT INTO CARD_REVIEWS (card_id, reviewed_at, rating, elapsed_seconds, scheduled_seconds, memory_before, memory_after, algorithm, parameters, desired_retention) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(review.card_id).bind(review.reviewed_at).bind(review.rating)
            .bind(review.elapsed_seconds).bind(review.scheduled_seconds)
            .bind(review.memory_before.map(sqlx::types::Json))
            .bind(sqlx::types::Json(review.memory_after)).bind(&review.algorithm)
            .bind(sqlx::types::Json(&review.parameters)).bind(review.desired_retention)
            .execute(&mut *conn).await?;
        Ok(sqlx::query_as("UPDATE CARDS SET memory_state = ?, last_reviewed_at = ?, due_at = ?, updated_at = ? WHERE id = ? RETURNING *")
            .bind(sqlx::types::Json(review.memory_after)).bind(review.reviewed_at)
            .bind(review.reviewed_at.checked_add(review.scheduled_seconds).ok_or_else(|| anyhow::anyhow!("复习时间溢出"))?)
            .bind(review.reviewed_at).bind(review.card_id).fetch_one(conn).await?)
    }

    pub async fn reset_memory_on(conn: &mut sqlx::SqliteConnection, id: i64) -> Result<()> {
        sqlx::query("UPDATE CARDS SET memory_state = NULL, last_reviewed_at = NULL, due_at = NULL, updated_at = ? WHERE id = ?")
            .bind(current_timestamp()).bind(id).execute(conn).await?;
        Ok(())
    }

    pub async fn review_history(&self, id: i64) -> Result<Vec<CardReview>> {
        Ok(
            sqlx::query_as("SELECT * FROM CARD_REVIEWS WHERE card_id = ? ORDER BY id ASC")
                .bind(id)
                .fetch_all(self.pool())
                .await?,
        )
    }
}
