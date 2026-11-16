use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 一张独立卡片；内容、模板和牌组分别通过 ID 关联。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, utoipa::ToSchema, FromRow)]
pub struct Card {
    pub id: i64,
    /// 卡片独占的内容记录，同一条 Note 不被多张卡片共享。
    pub note_id: i64,
    /// 引用独立的 TEMPLATES 表，多张卡片可以使用同一个模板。
    pub template_id: i64,
    pub deck_id: i64,
    /// FSRS-7 记忆状态；尚未进行有效评分的新卡为 None。
    #[serde(default)]
    #[sqlx(json(nullable))]
    pub memory_state: Option<StoredMemoryState>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 用于持久化的 FSRS-7 状态，不依赖算法库的序列化实现。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct StoredMemoryState {
    pub stability: f32,
    pub stability_fast: f32,
    pub difficulty: f32,
}

/// 创建卡片时需要的关联信息。
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateCard {
    pub note_id: i64,
    pub template_id: i64,
    pub deck_id: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::pool::SqlitePool;

    #[tokio::test]
    async fn card_requires_existing_relations_and_exclusive_note() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let pool = SqlitePool::new(dir.path().join("cards.db")).await?;
        pool.initialize_database().await?;
        sqlx::raw_sql(
            "INSERT INTO DECKS (id, name, config_id, created_at, updated_at) VALUES (1, 'Deck', 1, 0, 0);
             INSERT INTO NOTETYPES (id, name, fields) VALUES (1, 'Basic', '[]');
             INSERT INTO NOTES (id, note_type, content) VALUES (1, 1, '{}'), (2, 1, '{}');
             INSERT INTO TEMPLATES (id, name) VALUES (1, 'Basic');",
        )
        .execute(pool.inner())
        .await?;
        let card = sqlx::query_as::<_, Card>(
            "INSERT INTO CARDS (note_id, template_id, deck_id, created_at, updated_at)
             VALUES (1, 1, 1, 0, 0) RETURNING *",
        )
        .fetch_one(pool.inner())
        .await?;
        assert_eq!((card.note_id, card.template_id, card.deck_id), (1, 1, 1));
        assert_eq!(card.memory_state, None);

        let memory = StoredMemoryState {
            stability: 3.0,
            stability_fast: 2.4,
            difficulty: 5.0,
        };
        let updated =
            sqlx::query_as::<_, Card>("UPDATE CARDS SET memory_state = ? WHERE id = ? RETURNING *")
                .bind(sqlx::types::Json(memory))
                .bind(card.id)
                .fetch_one(pool.inner())
                .await?;
        assert_eq!(updated.memory_state, Some(memory));
        let json = serde_json::to_string(&updated)?;
        assert_eq!(serde_json::from_str::<Card>(&json)?, updated);

        let cleared = sqlx::query_as::<_, Card>(
            "UPDATE CARDS SET memory_state = NULL WHERE id = ? RETURNING *",
        )
        .bind(card.id)
        .fetch_one(pool.inner())
        .await?;
        assert_eq!(cleared.memory_state, None);

        for (note_id, template_id, deck_id) in [(1, 1, 1), (99, 1, 1), (2, 99, 1), (2, 1, 99)] {
            assert!(
                sqlx::query("INSERT INTO CARDS (note_id, template_id, deck_id, created_at, updated_at) VALUES (?, ?, ?, 0, 0)")
                    .bind(note_id).bind(template_id).bind(deck_id)
                    .execute(pool.inner()).await.is_err()
            );
        }
        // 不同内容可以复用同一个模板。
        sqlx::query("INSERT INTO CARDS (note_id, template_id, deck_id, created_at, updated_at) VALUES (2, 1, 1, 0, 0)")
            .execute(pool.inner()).await?;
        pool.inner().close().await;
        Ok(())
    }
}
