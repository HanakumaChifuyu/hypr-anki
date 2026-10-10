# 数据表定义：`DECKS`

## 1. 表概述

`DECKS` 表用于持久化存储 Anki 牌组的基本信息与树形父子层级关系。每个牌组可以拥有任意数量的子牌组，也可以作为根牌组存在。

* **对应实体**：[`stores::models::deck::Deck`](../../src/models/deck.rs)
* **仓储操作**：[`stores::repository::deck::DeckRepository`](../../src/repository/deck.rs)
* **所属迁移**：[`migrations/20261009071547_init_sql.sql`](../../../../migrations/20261009071547_init_sql.sql)

---

## 2. 建表 DDL 与索引

```sql
CREATE TABLE IF NOT EXISTS DECKS (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  is_collapsed INTEGER NOT NULL,
  parent_id INTEGER,
  config_id INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

-- 加速基于父节点的子牌组查找（树形展开/折叠渲染）
CREATE INDEX IF NOT EXISTS idx_decks_parent_id ON DECKS (parent_id);

-- 加速基于学习配置组的牌组过滤与分组
CREATE INDEX IF NOT EXISTS idx_decks_config_id ON DECKS (config_id);
```

---

## 3. 字段字典

| 列名 (Column) | SQLite 类型 | Rust 类型 | 允许为空 (Nullable) | 默认/取值约束 | 说明 (Description) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `id` | `INTEGER` | `i64` | 否 | 主键自增 | 牌组的唯一数字 ID |
| `name` | `TEXT` | `String` | 否 | 非空字符串 | **牌组的局部名称（叶子名）**。例如 `"Vocab"`，不包含父级前缀 |
| `is_collapsed` | `INTEGER` | `bool` | 否 | `0` 或 `1` | UI 树形界面折叠状态。`0` 表示展开，`1` 表示收起/折叠 |
| `parent_id` | `INTEGER` | `Option<i64>` | 是 | `NULL` / 外键 | 父牌组 ID。为 `NULL` 表示当前为根牌组；否则指向父牌组的 `id` |
| `config_id` | `INTEGER` | `i64` | 否 | 引用配置 ID | 绑定的学习配置组 ID（控制每日新卡上限、复习上限、FSRS 算法参数等） |
| `created_at` | `INTEGER` | `i64` | 否 | Unix 秒级时间戳 | 牌组的创建时间 |
| `updated_at` | `INTEGER` | `i64` | 否 | Unix 秒级时间戳 | 牌组元数据（名称、层级、配置等）的最后修改时间 |

---

## 4. 关键设计考量与决策

### 4.1 局部名称（Leaf Name）设计
* **区别于官方 Anki 纯字符串 `::` 方案**：官方 Anki 传统上将名称直接存为 `"Languages::Japanese::Vocab"`。
* **HyprAnki 的关系型树设计**：本项目通过 `parent_id` 显式维护树状父子层级，因此 `name` 仅保存当前节点本身的名称（如 `"Vocab"`）。
* **优势**：
  1. 重命名父牌组或拖拽迁移牌组位置时，无需通过复杂的字符串前缀匹配递归修改成百上千个后代节点的名称；
  2. 移动牌组只需单行更新：`UPDATE DECKS SET parent_id = ? WHERE id = ?`。

### 4.2 卡片学习状态统计字段不入库
* 今日待复习数（`review_count`）、待学新卡数（`new_count`）是随系统日期动态演变的运行时指标。
* 这些计数值通过 SQL 查询卡片表（`CARDS`）动态聚合获取，不在 `DECKS` 表中存储，避免缓存失效或并发脏读。
