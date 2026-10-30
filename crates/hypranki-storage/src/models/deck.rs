use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 数据库中的牌组记录 (DECKS 表实体)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct Deck {
    pub id: i64,
    pub name: String,
    pub is_collapsed: bool,
    pub parent_id: Option<i64>,
    pub config_id: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 创建牌组时的入参 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDeck {
    pub name: String,
    pub is_collapsed: bool,
    pub parent_id: Option<i64>,
    pub config_id: i64,
}

impl CreateDeck {
    pub fn new(name: impl Into<String>, config_id: i64) -> Self {
        Self {
            name: name.into(),
            is_collapsed: false,
            parent_id: None,
            config_id,
        }
    }

    pub fn with_parent(mut self, parent_id: Option<i64>) -> Self {
        self.parent_id = parent_id;
        self
    }

    pub fn with_collapsed(mut self, is_collapsed: bool) -> Self {
        self.is_collapsed = is_collapsed;
        self
    }
}

/// 更新牌组时的入参 DTO
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateDeck {
    pub name: Option<String>,
    pub is_collapsed: Option<bool>,
    /// None 表示不修改；Some(None) 表示设置为 NULL；Some(Some(id)) 表示更新为新的父 ID
    pub parent_id: Option<Option<i64>>,
    pub config_id: Option<i64>,
}

