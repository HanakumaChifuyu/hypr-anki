use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow, ensure};
use hypranki_storage::models::deck::{CreateDeck, Deck, UpdateDeck};
use hypranki_storage::repository::deck::DeckRepository;

#[derive(Debug, Clone)]
pub struct DeckService {
    deck_repo: Arc<DeckRepository>,
}

impl DeckService {
    pub fn new(deck_repo: Arc<DeckRepository>) -> Self {
        Self { deck_repo }
    }

    /// 从数据库获取完整树快照，按先序排列；前端用 parent_id 组装树。
    pub async fn list(&self) -> Result<Vec<Deck>> {
        let decks = self.deck_repo.find_all().await?;
        Ok(Self::to_preorder(&decks)?.into_iter().cloned().collect())
    }

    pub async fn get(&self, id: i64) -> Result<Deck> {
        self.deck_repo
            .find_by_id(id)
            .await
            .context(format!("查询牌组 {} 的时候发生数据库错误", id))?
            .ok_or_else(|| anyhow!("牌组 {} 不存在", id))
    }

    /// 校验和写入共用 SQLite 写事务，避免并发修改使校验失效。
    pub async fn create(&self, input: &CreateDeck) -> Result<Deck> {
        let mut tx = self.deck_repo.pool().begin_with("BEGIN IMMEDIATE").await?;
        let decks = DeckRepository::find_all_on(&mut tx).await?;
        Self::to_preorder(&decks)?;
        ensure!(!input.name.trim().is_empty(), "牌组名称不能为空");
        if let Some(parent_id) = input.parent_id {
            let parent = decks
                .iter()
                .find(|deck| deck.id == parent_id)
                .ok_or_else(|| anyhow!("父节点 {} 不存在", parent_id))?;
            ensure!(parent.folder, "父节点 {} 不是文件夹", parent_id);
        }
        let deck = DeckRepository::create_on(&mut tx, input).await?;
        tx.commit().await?;
        Ok(deck)
    }

    /// 部分更新：合并后校验整棵树，失败时不会写入。
    /// parent_id 为 Some(None) 时移到根目录，为 None 时保持原父节点。
    pub async fn update(&self, id: i64, input: &UpdateDeck) -> Result<Deck> {
        let mut tx = self.deck_repo.pool().begin_with("BEGIN IMMEDIATE").await?;
        let mut decks = DeckRepository::find_all_on(&mut tx).await?;
        Self::to_preorder(&decks)?;
        let deck = decks
            .iter_mut()
            .find(|deck| deck.id == id)
            .ok_or_else(|| anyhow!("牌组 {} 不存在", id))?;
        if let Some(name) = &input.name {
            ensure!(!name.trim().is_empty(), "牌组名称不能为空");
            deck.name = name.clone();
        }
        if let Some(folder) = input.folder {
            deck.folder = folder;
        }
        if let Some(parent_id) = input.parent_id {
            deck.parent_id = parent_id;
        }
        if let Some(config_id) = input.config_id {
            deck.config_id = config_id;
        }
        let updated = deck.clone();
        Self::to_preorder(&decks)?;
        let saved = DeckRepository::update_on(&mut tx, &updated)
            .await?
            .ok_or_else(|| anyhow!("牌组 {} 不存在", id))?;
        tx.commit().await?;
        Ok(saved)
    }

    pub async fn rename(&self, id: i64, name: impl Into<String>) -> Result<Deck> {
        self.update(
            id,
            &UpdateDeck {
                name: Some(name.into()),
                ..Default::default()
            },
        )
        .await
    }

    /// 移动整个子树；None 表示移到根目录。同级顺序按 ID 排列。
    pub async fn move_to(&self, id: i64, parent_id: Option<i64>) -> Result<Deck> {
        self.update(
            id,
            &UpdateDeck {
                parent_id: Some(parent_id),
                ..Default::default()
            },
        )
        .await
    }

    /// 删除节点及全部后代，返回删除的 ID，供前端同步移除节点。
    pub async fn delete_subtree(&self, id: i64) -> Result<Vec<i64>> {
        let mut tx = self.deck_repo.pool().begin_with("BEGIN IMMEDIATE").await?;
        let decks = DeckRepository::find_all_on(&mut tx).await?;
        let preorder = Self::to_preorder(&decks)?;
        ensure!(decks.iter().any(|deck| deck.id == id), "牌组 {} 不存在", id);
        let mut selected = std::collections::HashSet::from([id]);
        let mut deleted = Vec::new();
        for deck in preorder {
            if deck.id == id
                || deck
                    .parent_id
                    .is_some_and(|parent| selected.contains(&parent))
            {
                selected.insert(deck.id);
                deleted.push(deck.id);
            }
        }
        // 分批删除，避免超出 SQLite 的绑定参数数量上限。
        for batch in deleted.chunks(500) {
            DeckRepository::delete_by_ids_on(&mut tx, batch).await?;
        }
        tx.commit().await?;
        Ok(deleted)
    }

    /// 完整先序序列；同级按 ID 升序。空输入返回空序列。
    ///
    /// ID 重复、父节点不存在、父节点不是文件夹或父子关系存在环时返回错误。
    pub fn to_preorder(decks: &[Deck]) -> Result<Vec<&Deck>> {
        let mut by_id = HashMap::with_capacity(decks.len());
        for deck in decks {
            ensure!(
                by_id.insert(deck.id, deck).is_none(),
                "牌组 ID 重复：{}",
                deck.id
            );
        }
        for deck in decks {
            if let Some(parent_id) = deck.parent_id {
                ensure!(
                    by_id.contains_key(&parent_id),
                    "牌组 {} 的父节点 {} 不存在",
                    deck.id,
                    parent_id
                );
                ensure!(by_id[&parent_id].folder, "父节点 {} 不是文件夹", parent_id);
            }
        }

        let mut children: HashMap<Option<i64>, Vec<&Deck>> = HashMap::new();
        for deck in decks {
            children.entry(deck.parent_id).or_default().push(deck);
        }
        for siblings in children.values_mut() {
            siblings.sort_unstable_by_key(|deck| deck.id);
        }
        // 使用显式栈，避免很深的树耗尽调用栈。
        // ID 唯一且每个节点只有一个父节点，从根节点出发不会进入环。
        let mut stack: Vec<&Deck> = children
            .get(&None)
            .map(|roots| roots.iter().rev().copied().collect())
            .unwrap_or_default();
        let mut result = Vec::with_capacity(decks.len());
        while let Some(deck) = stack.pop() {
            result.push(deck);
            if let Some(nodes) = children.get(&Some(deck.id)) {
                stack.extend(nodes.iter().rev().copied());
            }
        }
        // 所有父节点均存在时，无法从根到达的分量必然包含环，
        // 包括自环、没有根节点的环，以及与正常树并存的独立环。
        ensure!(result.len() == decks.len(), "牌组父子关系存在环");
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use anyhow::Result;
    use hypranki_storage::db::pool::SqlitePool;
    use hypranki_storage::models::deck::Deck;
    use hypranki_storage::models::deck::{CreateDeck, UpdateDeck};
    use hypranki_storage::repository::deck::DeckRepository;

    use super::DeckService;

    async fn test_service() -> Result<(DeckService, tempfile::TempDir)> {
        let dir = tempfile::tempdir()?;
        let pool = SqlitePool::new(dir.path().join("decks.db")).await?;
        pool.migrate(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"))
            .await?;
        Ok((DeckService::new(Arc::new(DeckRepository::new(pool))), dir))
    }

    #[tokio::test]
    async fn tree_operations_persist_and_preserve_other_branches() -> Result<()> {
        let (service, _dir) = test_service().await?;
        let root = service
            .create(&CreateDeck::new("Root", 1).with_folder(true))
            .await?;
        let folder = service
            .create(
                &CreateDeck::new("Folder", 1)
                    .with_folder(true)
                    .with_parent(Some(root.id)),
            )
            .await?;
        let leaf = service
            .create(&CreateDeck::new("Leaf", 1).with_parent(Some(folder.id)))
            .await?;
        let other = service.create(&CreateDeck::new("Other", 1)).await?;
        let renamed = service.rename(leaf.id, "Renamed").await?;
        assert_eq!(service.get(leaf.id).await?, renamed);
        service.move_to(folder.id, None).await?;
        assert_eq!(service.get(leaf.id).await?.parent_id, Some(folder.id));
        let ids: Vec<_> = service.list().await?.iter().map(|deck| deck.id).collect();
        assert_eq!(ids, vec![root.id, folder.id, leaf.id, other.id]);
        assert_eq!(
            service.delete_subtree(folder.id).await?,
            vec![folder.id, leaf.id]
        );
        assert!(service.get(folder.id).await.is_err());
        assert!(service.get(leaf.id).await.is_err());
        assert_eq!(service.list().await?, vec![root, other]);
        Ok(())
    }

    #[tokio::test]
    async fn invalid_mutations_leave_database_unchanged() -> Result<()> {
        let (service, _dir) = test_service().await?;
        let root = service
            .create(&CreateDeck::new("Root", 1).with_folder(true))
            .await?;
        let child = service
            .create(
                &CreateDeck::new("Child", 1)
                    .with_folder(true)
                    .with_parent(Some(root.id)),
            )
            .await?;
        let leaf = service.create(&CreateDeck::new("Leaf", 1)).await?;
        let before = service.list().await?;
        assert!(
            service
                .create(&CreateDeck::new("Missing", 1).with_parent(Some(999)))
                .await
                .is_err()
        );
        assert!(
            service
                .create(&CreateDeck::new("Hidden", 1).with_parent(Some(leaf.id)))
                .await
                .is_err()
        );
        assert!(service.create(&CreateDeck::new("  ", 1)).await.is_err());
        for parent in [Some(root.id), Some(child.id), Some(leaf.id), Some(999)] {
            assert!(service.move_to(root.id, parent).await.is_err());
        }
        assert!(
            service
                .update(
                    root.id,
                    &UpdateDeck {
                        name: Some("Should roll back".into()),
                        folder: Some(false),
                        ..Default::default()
                    }
                )
                .await
                .is_err()
        );
        assert!(service.rename(leaf.id, "\t").await.is_err());
        assert!(service.rename(999, "Missing").await.is_err());
        assert!(service.delete_subtree(999).await.is_err());
        assert_eq!(service.list().await?, before);
        // 空文件夹可以转成普通牌组。
        assert!(
            !service
                .update(
                    child.id,
                    &UpdateDeck {
                        folder: Some(false),
                        ..Default::default()
                    }
                )
                .await?
                .folder
        );
        Ok(())
    }

    #[tokio::test]
    async fn concurrent_moves_cannot_create_cycle() -> Result<()> {
        let (service, _dir) = test_service().await?;
        let a = service
            .create(&CreateDeck::new("A", 1).with_folder(true))
            .await?;
        let b = service
            .create(&CreateDeck::new("B", 1).with_folder(true))
            .await?;
        let (first, second) = tokio::join!(
            service.move_to(a.id, Some(b.id)),
            service.move_to(b.id, Some(a.id))
        );
        assert_ne!(first.is_ok(), second.is_ok());
        assert_eq!(service.list().await?.len(), 2);
        Ok(())
    }

    #[test]
    fn preorder_rejects_children_of_normal_deck() {
        let mut decks = decks_with_parents(&[None, Some(1)]);
        decks[0].folder = false;
        assert_eq!(
            DeckService::to_preorder(&decks).unwrap_err().to_string(),
            "父节点 1 不是文件夹"
        );
    }

    /// Mock 邻接表，输入刻意打乱，子节点可能出现在父节点之前。
    /// 同级节点按 ID 升序排列时，对应的树为：
    ///
    /// ```text
    /// 1 英语
    /// ├── 2 词汇
    /// │   ├── 3 日常词汇
    /// │   │   └── 4 食物
    /// │   └── 5 学术词汇
    /// └── 6 语法
    ///     └── 7 时态
    /// 8 编程
    /// ├── 9 Rust
    /// │   └── 10 所有权
    /// └── 11 SQL
    /// 12 收件箱
    /// ```

    fn mock_deck_table() -> Vec<Deck> {
        let rows = [
            (10, "所有权", Some(9), false),
            (5, "学术词汇", Some(2), false),
            (8, "编程", None, false),
            (3, "日常词汇", Some(2), false),
            (12, "收件箱", None, false),
            (7, "时态", Some(6), false),
            (1, "英语", None, false),
            (11, "SQL", Some(8), false),
            (4, "食物", Some(3), false),
            (9, "Rust", Some(8), true),
            (6, "语法", Some(1), false),
            (2, "词汇", Some(1), false),
        ];

        rows.into_iter()
            .map(|(id, name, parent_id, folder)| Deck {
                id,
                name: name.to_owned(),
                folder: folder || matches!(id, 1 | 2 | 3 | 6 | 8 | 9),
                parent_id,
                config_id: 1,
                created_at: 1_700_000_000,
                updated_at: 1_700_000_000,
            })
            .collect()
    }

    // 同级按 ID 升序遍历时的预期结果，可用于转换方法的 assert_eq!。
    const EXPECTED_PREORDER_IDS: [i64; 12] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

    #[test]
    fn test_topreorder() {
        let decks = mock_deck_table();
        let preorder_decks = DeckService::to_preorder(&decks).unwrap();
        let idx: Vec<_> = preorder_decks.iter().map(|&deck| deck.id).collect();
        assert_eq!(idx, EXPECTED_PREORDER_IDS);
    }

    fn decks_with_parents(parents: &[Option<i64>]) -> Vec<Deck> {
        let prototype = mock_deck_table().remove(0);
        parents
            .iter()
            .enumerate()
            .map(|(index, &parent_id)| Deck {
                id: index as i64 + 1,
                parent_id,
                folder: true,
                ..prototype.clone()
            })
            .collect()
    }

    #[test]
    fn preorder_accepts_empty_input() {
        assert!(DeckService::to_preorder(&[]).unwrap().is_empty());
    }

    #[test]
    fn preorder_rejects_missing_parent() {
        let decks = decks_with_parents(&[None, Some(99)]);
        let error = DeckService::to_preorder(&decks).unwrap_err();
        assert_eq!(error.to_string(), "牌组 2 的父节点 99 不存在");
    }

    #[test]
    fn preorder_rejects_duplicate_ids() {
        let mut decks = decks_with_parents(&[None, None]);
        decks[1].id = decks[0].id;
        let error = DeckService::to_preorder(&decks).unwrap_err();
        assert_eq!(error.to_string(), "牌组 ID 重复：1");
    }

    #[test]
    fn preorder_rejects_cycles() {
        for parents in [
            vec![Some(1)],
            vec![Some(2), Some(1)],
            vec![None, Some(3), Some(2), Some(3)],
        ] {
            let decks = decks_with_parents(&parents);
            let error = DeckService::to_preorder(&decks).unwrap_err();
            assert_eq!(error.to_string(), "牌组父子关系存在环");
        }
    }

    #[test]
    fn preorder_accepts_deep_tree() {
        let mut parents = vec![None];
        parents.extend((1..10_000).map(Some));
        let decks = decks_with_parents(&parents);
        let preorder = DeckService::to_preorder(&decks).unwrap();
        let ids: Vec<_> = preorder.iter().map(|deck| deck.id).collect();
        assert_eq!(ids, (1..=10_000).collect::<Vec<_>>());
    }
}
