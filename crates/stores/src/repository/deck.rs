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
        Self { pool: pool.clone() }
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
    use anyhow::Result;
    use std::path::Path;
    use tempfile::{TempDir, tempdir};

    use super::*;

    async fn create_test_deck_repository() -> Result<(DeckRepository, TempDir)> {
        let temp_dir = tempdir()?;
        let test_db = temp_dir.path().join("test.db");
        let temp_pool = SqlitePool::new(test_db).await?;
        let migr_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations");
        temp_pool.migrate(migr_dir).await?;
        let repository = DeckRepository::new(temp_pool);
        Ok((repository, temp_dir))
    }

    /// 1. 测试基础增删查改与存在性校验 (Create, find_by_id, find_by_name, exists_by_id, count)
    #[tokio::test]
    async fn test_deck_crud() -> Result<()> {
        let (test_repo, _guard) = create_test_deck_repository().await?;

        // 初始数量应为 0
        assert_eq!(test_repo.count().await?, 0);

        // 新建 deck
        let test_createdeck1 = CreateDeck::new("test_deck1", 1);
        let test_createdeck2 = CreateDeck::new("test_deck2", 1);
        let deck1 = test_repo.create(&test_createdeck1).await?;
        let deck2 = test_repo.create(&test_createdeck2).await?;

        assert_eq!(deck1.name, test_createdeck1.name);
        assert_eq!(deck1.config_id, test_createdeck1.config_id);
        assert_eq!(deck1.is_collapsed, false);
        assert_eq!(deck1.parent_id, None);

        // 验证 count 增加
        assert_eq!(test_repo.count().await?, 2);

        // 测试按 id 查找
        let found2 = test_repo.find_by_id(deck2.id).await?;
        assert!(found2.is_some());
        let found2 = found2.unwrap();
        assert_eq!(found2.name, test_createdeck2.name);
        assert_eq!(found2.config_id, test_createdeck2.config_id);

        // 测试查找不存在的 id
        let not_found = test_repo.find_by_id(9999).await?;
        assert!(not_found.is_none());

        // 测试 exists_by_id
        assert!(test_repo.exists_by_id(deck1.id).await?);
        assert!(!test_repo.exists_by_id(9999).await?);

        // 测试 find_by_name
        let found_name = test_repo.find_by_name("test_deck1").await?;
        assert_eq!(found_name, Some(deck1));

        let not_found_name = test_repo.find_by_name("non_existent").await?;
        assert_eq!(not_found_name, None);

        // 测试 create_simple
        let simple_deck = test_repo.create_simple("simple_deck", None, 2).await?;
        assert_eq!(simple_deck.name, "simple_deck");
        assert_eq!(simple_deck.config_id, 2);

        Ok(())
    }

    /// 2. 测试层级与父子关系查询 (find_all, find_root_decks, find_by_parent_id)
    #[tokio::test]
    async fn test_deck_hierarchy_queries() -> Result<()> {
        let (repo, _guard) = create_test_deck_repository().await?;

        // 两个根牌组
        let root1 = repo.create_simple("Languages", None, 1).await?;
        let root2 = repo.create_simple("Science", None, 1).await?;

        // Languages 的子牌组
        let sub1 = repo
            .create(&CreateDeck::new("Japanese", 1).with_parent(Some(root1.id)))
            .await?;
        let sub2 = repo
            .create(&CreateDeck::new("Chinese", 1).with_parent(Some(root1.id)))
            .await?;

        // find_all 应包含所有 4 个牌组
        let all = repo.find_all().await?;
        assert_eq!(all.len(), 4);

        // find_root_decks 应只返回两个根牌组
        let roots = repo.find_root_decks().await?;
        assert_eq!(roots.len(), 2);
        assert!(roots.iter().any(|d| d.id == root1.id));
        assert!(roots.iter().any(|d| d.id == root2.id));

        // find_by_parent_id(Some(root1.id)) 应只返回 2 个子牌组
        let lang_children = repo.find_by_parent_id(Some(root1.id)).await?;
        assert_eq!(lang_children.len(), 2);
        assert!(lang_children.iter().any(|d| d.id == sub1.id));
        assert!(lang_children.iter().any(|d| d.id == sub2.id));

        // find_by_parent_id(Some(root2.id)) 应为空
        let science_children = repo.find_by_parent_id(Some(root2.id)).await?;
        assert!(science_children.is_empty());

        // find_by_parent_id(None) 等同于 find_root_decks
        let none_children = repo.find_by_parent_id(None).await?;
        assert_eq!(none_children.len(), 2);

        Ok(())
    }

    /// 3. 测试各类更新方法 (update, update_name, update_collapsed, update_parent, update_config)
    #[tokio::test]
    async fn test_deck_updates() -> Result<()> {
        let (repo, _guard) = create_test_deck_repository().await?;
        let mut deck = repo.create_simple("Original", None, 1).await?;

        // update_name
        let updated = repo.update_name(deck.id, "Renamed").await?;
        assert!(updated);
        let current = repo.find_by_id(deck.id).await?.unwrap();
        assert_eq!(current.name, "Renamed");

        // update_collapsed
        let updated = repo.update_collapsed(deck.id, true).await?;
        assert!(updated);
        let current = repo.find_by_id(deck.id).await?.unwrap();
        assert!(current.is_collapsed);

        // update_parent
        let parent = repo.create_simple("Parent", None, 1).await?;
        let updated = repo.update_parent(deck.id, Some(parent.id)).await?;
        assert!(updated);
        let current = repo.find_by_id(deck.id).await?.unwrap();
        assert_eq!(current.parent_id, Some(parent.id));

        // update_config
        let updated = repo.update_config(deck.id, 99).await?;
        assert!(updated);
        let current = repo.find_by_id(deck.id).await?.unwrap();
        assert_eq!(current.config_id, 99);

        // 全量 update
        deck.name = "FullUpdate".to_string();
        deck.is_collapsed = false;
        deck.parent_id = None;
        deck.config_id = 200;
        let updated = repo.update(&deck).await?;
        assert!(updated);

        let current = repo.find_by_id(deck.id).await?.unwrap();
        assert_eq!(current.name, "FullUpdate");
        assert!(!current.is_collapsed);
        assert_eq!(current.parent_id, None);
        assert_eq!(current.config_id, 200);

        // 更新不存在的 ID 应返回 false
        assert!(!repo.update_name(9999, "None").await?);
        assert!(!repo.update_collapsed(9999, true).await?);
        assert!(!repo.update_parent(9999, None).await?);
        assert!(!repo.update_config(9999, 1).await?);

        Ok(())
    }

    /// 4. 测试动态部分更新 (update_partial)
    #[tokio::test]
    async fn test_deck_update_partial() -> Result<()> {
        let (repo, _guard) = create_test_deck_repository().await?;
        let parent = repo.create_simple("Parent", None, 1).await?;
        let deck = repo
            .create(&CreateDeck::new("TestPartial", 1).with_parent(Some(parent.id)))
            .await?;

        // 仅更新名称，其他属性保持原有值
        let partial_update = UpdateDeck {
            name: Some("PartialRenamed".to_string()),
            ..Default::default()
        };
        let res = repo.update_partial(deck.id, &partial_update).await?;
        assert!(res.is_some());
        let updated = res.unwrap();
        assert_eq!(updated.name, "PartialRenamed");
        assert_eq!(updated.parent_id, Some(parent.id));
        assert_eq!(updated.config_id, 1);

        // 将 parent_id 置为 NULL (Some(None))
        let detach_parent = UpdateDeck {
            parent_id: Some(None),
            ..Default::default()
        };
        let res = repo.update_partial(deck.id, &detach_parent).await?;
        assert!(res.is_some());
        let updated = res.unwrap();
        assert_eq!(updated.parent_id, None);

        // 更新不存在的 ID 返回 None
        let not_found = repo
            .update_partial(9999, &UpdateDeck {
                name: Some("Ghost".to_string()),
                ..Default::default()
            })
            .await?;
        assert!(not_found.is_none());

        Ok(())
    }

    /// 5. 测试单个与批量删除 (delete_by_id, delete_by_ids)
    #[tokio::test]
    async fn test_deck_deletions() -> Result<()> {
        let (repo, _guard) = create_test_deck_repository().await?;
        let d1 = repo.create_simple("Deck 1", None, 1).await?;
        let d2 = repo.create_simple("Deck 2", None, 1).await?;
        let d3 = repo.create_simple("Deck 3", None, 1).await?;
        let d4 = repo.create_simple("Deck 4", None, 1).await?;

        assert_eq!(repo.count().await?, 4);

        // delete_by_id 成功删除
        let deleted = repo.delete_by_id(d1.id).await?;
        assert!(deleted);
        assert!(!repo.exists_by_id(d1.id).await?);
        assert_eq!(repo.count().await?, 3);

        // delete_by_id 删除不存在的 ID
        let not_deleted = repo.delete_by_id(9999).await?;
        assert!(!not_deleted);

        // delete_by_ids 空数组返回 0
        let affected = repo.delete_by_ids(&[]).await?;
        assert_eq!(affected, 0);

        // delete_by_ids 批量删除 d2 和 d3
        let affected = repo.delete_by_ids(&[d2.id, d3.id]).await?;
        assert_eq!(affected, 2);
        assert_eq!(repo.count().await?, 1);
        assert!(repo.exists_by_id(d4.id).await?);

        Ok(())
    }
}
