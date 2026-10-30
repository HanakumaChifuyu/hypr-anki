use std::collections::HashMap;

use hypranki_storage::models::deck::Deck;
use hypranki_storage::repository::deck;

struct DeckService {
    deck_repo: deck::DeckRepository,
}

impl DeckService {
    pub fn to_preorder(decks: &Vec<Deck>) -> Vec<&Deck> {
        // 邻接表
        let mut children: HashMap<Option<i64>, Vec<&Deck>> = HashMap::new();
        for deck in decks {
            children.entry(deck.parent_id).or_default().push(deck);
        }
        for siblings in children.values_mut() {
            siblings.sort_unstable_by_key(|deck| deck.id);
        }

        fn visit<'a>(
            parent_id: Option<i64>,
            children: &HashMap<Option<i64>, Vec<&'a Deck>>,
            result: &mut Vec<&'a Deck>,
        ) {
            if let Some(siblings) = children.get(&parent_id) {
                for &deck in siblings {
                    result.push(deck);
                    visit(Some(deck.id), children, result);
                }
            }
        }

        let mut result: Vec<&Deck> = Vec::new();
        visit(None, &children, &mut result);
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
