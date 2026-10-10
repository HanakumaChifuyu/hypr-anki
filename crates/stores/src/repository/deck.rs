use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use sqlx::{Pool, Sqlite};

use crate::db::pool::SqlitePool;
pub use crate::models::deck::{CreateDeck, Deck, UpdateDeck};

fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[derive(Debug, Clone)]
pub struct DeckRepository {
    pool: SqlitePool,
}

impl DeckRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 获取底层数据库连接池
    pub fn pool(&self) -> &Pool<Sqlite> {
        self.pool.inner()
    }

    // ==================== 增 (Create) ====================

    /// 创建一个新牌组
    pub async fn create(&self, input: &CreateDeck) -> Result<Deck> {
        let now = current_timestamp();
        let deck = sqlx::query_as::<_, Deck>(
            r#"
            INSERT INTO DECKS (name, is_collapsed, parent_id, config_id, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING id, name, is_collapsed, parent_id, config_id, created_at, updated_at
            "#,
        )
        .bind(&input.name)
        .bind(input.is_collapsed)
        .bind(input.parent_id)
        .bind(input.config_id)
        .bind(now)
        .bind(now)
        .fetch_one(self.pool())
        .await?;

        Ok(deck)
    }

    /// 快捷创建普通根牌组
    pub async fn create_simple(
        &self,
        name: impl Into<String>,
        parent_id: Option<i64>,
        config_id: i64,
    ) -> Result<Deck> {
        let input = CreateDeck {
            name: name.into(),
            is_collapsed: false,
            parent_id,
            config_id,
        };
        self.create(&input).await
    }

    // ==================== 查 (Read) ====================

    /// 根据 ID 查询单个牌组
    pub async fn find_by_id(&self, id: i64) -> Result<Option<Deck>> {
        let deck = sqlx::query_as::<_, Deck>(
            r#"
            SELECT id, name, is_collapsed, parent_id, config_id, created_at, updated_at
            FROM DECKS
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?;

        Ok(deck)
    }

    /// 查询所有牌组（默认按 id 升序）
    pub async fn find_all(&self) -> Result<Vec<Deck>> {
        let decks = sqlx::query_as::<_, Deck>(
            r#"
            SELECT id, name, is_collapsed, parent_id, config_id, created_at, updated_at
            FROM DECKS
            ORDER BY id ASC
            "#,
        )
        .fetch_all(self.pool())
        .await?;

        Ok(decks)
    }

    /// 根据父牌组 ID 查询直接子牌组；若 parent_id 为 None 则查询所有顶层根牌组
    pub async fn find_by_parent_id(&self, parent_id: Option<i64>) -> Result<Vec<Deck>> {
        let decks = match parent_id {
            Some(pid) => {
                sqlx::query_as::<_, Deck>(
                    r#"
                    SELECT id, name, is_collapsed, parent_id, config_id, created_at, updated_at
                    FROM DECKS
                    WHERE parent_id = ?
                    ORDER BY id ASC
                    "#,
                )
                .bind(pid)
                .fetch_all(self.pool())
                .await?
            }
            None => {
                sqlx::query_as::<_, Deck>(
                    r#"
                    SELECT id, name, is_collapsed, parent_id, config_id, created_at, updated_at
                    FROM DECKS
                    WHERE parent_id IS NULL
                    ORDER BY id ASC
                    "#,
                )
                .fetch_all(self.pool())
                .await?
            }
        };

        Ok(decks)
    }

    /// 查询所有根牌组 (parent_id IS NULL)
    pub async fn find_root_decks(&self) -> Result<Vec<Deck>> {
        self.find_by_parent_id(None).await
    }

    /// 根据牌组名称查询（精确匹配）
    pub async fn find_by_name(&self, name: &str) -> Result<Option<Deck>> {
        let deck = sqlx::query_as::<_, Deck>(
            r#"
            SELECT id, name, is_collapsed, parent_id, config_id, created_at, updated_at
            FROM DECKS
            WHERE name = ?
            LIMIT 1
            "#,
        )
        .bind(name)
        .fetch_optional(self.pool())
        .await?;

        Ok(deck)
    }

    /// 检查指定 ID 的牌组是否存在
    pub async fn exists_by_id(&self, id: i64) -> Result<bool> {
        let row: Option<(i64,)> = sqlx::query_as(
            r#"
            SELECT 1 FROM DECKS WHERE id = ? LIMIT 1
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?;

        Ok(row.is_some())
    }

    /// 获取牌组总数
    pub async fn count(&self) -> Result<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM DECKS")
            .fetch_one(self.pool())
            .await?;
        Ok(row.0)
    }

    // ==================== 改 (Update) ====================

    /// 全量更新牌组信息
    pub async fn update(&self, deck: &Deck) -> Result<bool> {
        let now = current_timestamp();
        let result = sqlx::query(
            r#"
            UPDATE DECKS
            SET name = ?, is_collapsed = ?, parent_id = ?, config_id = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&deck.name)
        .bind(deck.is_collapsed)
        .bind(deck.parent_id)
        .bind(deck.config_id)
        .bind(now)
        .bind(deck.id)
        .execute(self.pool())
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 修改牌组名称
    pub async fn update_name(&self, id: i64, name: &str) -> Result<bool> {
        let now = current_timestamp();
        let result = sqlx::query(
            r#"
            UPDATE DECKS
            SET name = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(name)
        .bind(now)
        .bind(id)
        .execute(self.pool())
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 修改折叠状态
    pub async fn update_collapsed(&self, id: i64, is_collapsed: bool) -> Result<bool> {
        let now = current_timestamp();
        let result = sqlx::query(
            r#"
            UPDATE DECKS
            SET is_collapsed = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(is_collapsed)
        .bind(now)
        .bind(id)
        .execute(self.pool())
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 移动牌组（更换父牌组）
    pub async fn update_parent(&self, id: i64, parent_id: Option<i64>) -> Result<bool> {
        let now = current_timestamp();
        let result = sqlx::query(
            r#"
            UPDATE DECKS
            SET parent_id = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(parent_id)
        .bind(now)
        .bind(id)
        .execute(self.pool())
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 更换配置组
    pub async fn update_config(&self, id: i64, config_id: i64) -> Result<bool> {
        let now = current_timestamp();
        let result = sqlx::query(
            r#"
            UPDATE DECKS
            SET config_id = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(config_id)
        .bind(now)
        .bind(id)
        .execute(self.pool())
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 按需动态更新（支持部分字段更新）
    pub async fn update_partial(&self, id: i64, update: &UpdateDeck) -> Result<Option<Deck>> {
        let current = match self.find_by_id(id).await? {
            Some(d) => d,
            None => return Ok(None),
        };

        let new_name = update.name.clone().unwrap_or(current.name);
        let new_is_collapsed = update.is_collapsed.unwrap_or(current.is_collapsed);
        let new_parent_id = match update.parent_id {
            Some(pid) => pid,
            None => current.parent_id,
        };
        let new_config_id = update.config_id.unwrap_or(current.config_id);
        let now = current_timestamp();

        let updated = sqlx::query_as::<_, Deck>(
            r#"
            UPDATE DECKS
            SET name = ?, is_collapsed = ?, parent_id = ?, config_id = ?, updated_at = ?
            WHERE id = ?
            RETURNING id, name, is_collapsed, parent_id, config_id, created_at, updated_at
            "#,
        )
        .bind(&new_name)
        .bind(new_is_collapsed)
        .bind(new_parent_id)
        .bind(new_config_id)
        .bind(now)
        .bind(id)
        .fetch_optional(self.pool())
        .await?;

        Ok(updated)
    }

    // ==================== 删 (Delete) ====================

    /// 根据 ID 删除单个牌组
    pub async fn delete_by_id(&self, id: i64) -> Result<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM DECKS
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(self.pool())
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 批量删除牌组
    pub async fn delete_by_ids(&self, ids: &[i64]) -> Result<u64> {
        if ids.is_empty() {
            return Ok(0);
        }

        let mut builder = sqlx::QueryBuilder::new("DELETE FROM DECKS WHERE id IN (");
        let mut separated = builder.separated(", ");
        for id in ids {
            separated.push_bind(id);
        }
        separated.push_unseparated(")");

        let result = builder.build().execute(self.pool()).await?;
        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn setup_test_db() -> Result<(DeckRepository, tempfile::TempDir)> {
        let tempdir = tempdir()?;
        let db_path = tempdir.path().join("test_deck.db");
        let pool = SqlitePool::new(db_path).await?;

        // 运行 migrations 执行全套数据库迁移
        sqlx::migrate!("../../migrations").run(pool.inner()).await?;

        Ok((DeckRepository::new(pool), tempdir))
    }

    #[tokio::test]
    async fn test_deck_crud() -> Result<()> {
        let (repo, _tempdir) = setup_test_db().await?;

        // 1. 测试创建根牌组 (Create)
        let root = repo.create_simple("Default", None, 1).await?;
        assert_eq!(root.id, 1);
        assert_eq!(root.name, "Default");
        assert!(!root.is_collapsed);
        assert_eq!(root.parent_id, None);
        assert_eq!(root.config_id, 1);

        // 2. 测试创建子牌组
        let child_input = CreateDeck::new("Vocab", 1)
            .with_parent(Some(root.id))
            .with_collapsed(true);
        let child = repo.create(&child_input).await?;
        assert_eq!(child.id, 2);
        assert_eq!(child.name, "Vocab");
        assert!(child.is_collapsed);
        assert_eq!(child.parent_id, Some(root.id));

        // 3. 测试查询 (Read)
        let found = repo.find_by_id(root.id).await?;
        assert_eq!(found.unwrap().name, "Default");

        let not_found = repo.find_by_id(999).await?;
        assert!(not_found.is_none());

        let all_decks = repo.find_all().await?;
        assert_eq!(all_decks.len(), 2);

        let root_decks = repo.find_root_decks().await?;
        assert_eq!(root_decks.len(), 1);
        assert_eq!(root_decks[0].name, "Default");

        let children = repo.find_by_parent_id(Some(root.id)).await?;
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "Vocab");

        let found_by_name = repo.find_by_name("Default").await?;
        assert_eq!(found_by_name.unwrap().id, root.id);

        assert!(repo.exists_by_id(root.id).await?);
        assert!(!repo.exists_by_id(999).await?);
        assert_eq!(repo.count().await?, 2);

        // 4. 测试更新 (Update)
        // 更新名称
        let updated = repo.update_name(child.id, "N2").await?;
        assert!(updated);
        let reloaded_child = repo.find_by_id(child.id).await?.unwrap();
        assert_eq!(reloaded_child.name, "N2");

        // 更新折叠状态
        repo.update_collapsed(child.id, false).await?;
        let reloaded_child = repo.find_by_id(child.id).await?.unwrap();
        assert!(!reloaded_child.is_collapsed);

        // 移动牌组（解除父牌组关系，变为根牌组）
        repo.update_parent(child.id, None).await?;
        let root_decks_now = repo.find_root_decks().await?;
        assert_eq!(root_decks_now.len(), 2);

        // 部分更新 (update_partial)
        let update_dto = UpdateDeck {
            name: Some("Advanced".into()),
            config_id: Some(2),
            ..Default::default()
        };
        let partially_updated = repo.update_partial(child.id, &update_dto).await?;
        assert!(partially_updated.is_some());
        let u = partially_updated.unwrap();
        assert_eq!(u.name, "Advanced");
        assert_eq!(u.config_id, 2);

        // 5. 测试删除 (Delete)
        let deleted = repo.delete_by_id(child.id).await?;
        assert!(deleted);
        assert_eq!(repo.count().await?, 1);

        // 批量删除
        let deleted_batch = repo.delete_by_ids(&[root.id, 999]).await?;
        assert_eq!(deleted_batch, 1);
        assert_eq!(repo.count().await?, 0);

        Ok(())
    }
}

