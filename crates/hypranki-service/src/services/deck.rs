use std::collections::{HashMap, HashSet};

use anyhow::{Result, anyhow, ensure};
use hypranki_storage::models::deck::{CreateDeck, Deck};
use hypranki_storage::repository::deck::DeckRepository;

#[derive(Debug, Clone)]
pub struct DeckService {
    deck_repo: DeckRepository,
}

impl DeckService {
    pub fn new(deck_repo: DeckRepository) -> Self {
        Self { deck_repo }
    }

    /// 完整先序序列；同级按 ID 升序，忽略折叠状态。
    pub fn to_preorder(decks: &[Deck]) -> Vec<&Deck> {
        Self::traverse(decks, None, false)
    }

    /// 仅返回可见节点，折叠节点自身仍保留。
    pub fn to_visible_preorder(decks: &[Deck]) -> Vec<&Deck> {
        Self::traverse(decks, None, true)
    }

    // 使用显式栈，避免很深的树耗尽调用栈。异常环中的节点最多访问一次。
    fn traverse(decks: &[Deck], root: Option<i64>, visible_only: bool) -> Vec<&Deck> {
        let mut children: HashMap<Option<i64>, Vec<&Deck>> = HashMap::new();
        for deck in decks {
            children.entry(deck.parent_id).or_default().push(deck);
        }
        for siblings in children.values_mut() {
            siblings.sort_unstable_by_key(|deck| deck.id);
        }
        let mut stack: Vec<&Deck> = match root {
            Some(id) => decks.iter().find(|d| d.id == id).into_iter().collect(),
            None => children
                .get(&None)
                .map(|roots| roots.iter().rev().copied().collect())
                .unwrap_or_default(),
        };
        let mut visited = HashSet::new();
        let mut result = Vec::new();
        while let Some(deck) = stack.pop() {
            if !visited.insert(deck.id) {
                continue;
            }
            result.push(deck);
            if !(visible_only && deck.is_collapsed) {
                if let Some(nodes) = children.get(&Some(deck.id)) {
                    stack.extend(nodes.iter().rev().copied());
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use hypranki_storage::models::deck::Deck;

    use super::DeckService;

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
    /// `is_collapsed` 仅是展示状态；完整先序遍历仍应包含其后代。

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
            .map(|(id, name, parent_id, is_collapsed)| Deck {
                id,
                name: name.to_owned(),
                is_collapsed,
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
        let preorder_decks = DeckService::to_preorder(&decks);
        let idx: Vec<_> = preorder_decks.iter().map(|&deck| deck.id).collect();
        assert_eq!(idx, EXPECTED_PREORDER_IDS);
    }
}
