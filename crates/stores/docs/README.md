# Stores 数据存储层文档

本文档用于规范 `crates/stores` 中的数据模型演进、数据库表定义与仓储层实现。

---

## 目录表 (Table of Contents)

### 1. 数据表定义一览 (Database Tables)

| 表名 (Table) | 描述 (Description) | 详细文档 (Details) | 对应 Model | 对应 Repository |
| :--- | :--- | :--- | :--- | :--- |
| **`DECKS`** | 牌组表（基础元数据、树形父子关系、折叠状态） | [decks.md](./tables/decks.md) | `models::deck::Deck` | `repository::deck::DeckRepository` |

> 随着后续业务扩展，新增的表（如 `CARDS`、`MODELS`、`REVIEWS`、`DECK_CONFIGS` 等）请在此登记。

---

## 标准开发工作流程 (Development Workflow)

每次新增或修改数据库实体时，请严格遵守以下 4 步标准化流程：

```
[1. 设计结构体 Model] ──► [2. 追加建表 DDL] ──► [3. 实现仓储 CRUD & 单元测试] ──► [4. 补全表结构文档]
```

### 1. 先设计结构体 (Models & DTOs)
* **位置**：`crates/stores/src/models/<entity>.rs`
* **规范**：
  * 定义用于 1:1 映射数据表的扁平实体结构体（如 `Deck`），派生 `#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]`。
  * 分离入参 DTO：创建专用的 `Create<Entity>` 和 `Update<Entity>` 结构体，提供清晰的构造辅助方法（如 `.with_parent(...)`）。
  * **注意**：此处只设计数据库存储模型，不混入上层复杂的领域方法或 UI 统计字段。

### 2. 在迁移脚本中追加建表语句 (Migrations)
* **位置**：`migrations/20261009071547_init_sql.sql`（或新增的版本迁移文件）
* **规范**：
  * 使用标准的 SQLite DDL 语句：`CREATE TABLE IF NOT EXISTS <TABLE_NAME> (...)`。
  * 主键统一使用 `id INTEGER PRIMARY KEY AUTOINCREMENT`。
  * 必须显式为高频查询的外键字段追加索引（如 `CREATE INDEX IF NOT EXISTS idx_...`）。

### 3. 写好基本的增删改查方法 (Repository & Unit Tests)
* **位置**：`crates/stores/src/repository/<entity>.rs`
* **规范**：
  * 构造标准结构体 `<Entity>Repository`，持有包装层 `SqlitePool`。
  * 实现核心 CRUD：创建 (`create`, `create_simple`)、查询 (`find_by_id`, `find_all`, 按特定条件查询)、更新 (`update`, `update_partial`, 精准字段更新)、删除 (`delete_by_id`, `delete_by_ids`)。
  * **编写单元测试**：Repository 层的测试直接在**同文件末尾的 `#[cfg(test)] mod tests`** 中作为单元测试编写，**统一使用 `sqlx::migrate!`** 执行数据库初始化，无需放入外部 `tests/` 集成测试目录。

### 4. 补全文档内容 (Documentation)
* **位置**：`crates/stores/docs/tables/<table_name>.md`
* **规范**：
  * 记录表名、表用途、完整建表 DDL。
  * 逐列列出字段名称、类型、空值约束、默认值以及业务释义。
  * 说明关键设计决策（如命名策略、反范式考虑等）。
  * 在当前 `docs/README.md` 的“数据表定义一览”中登记链接。

---

## 核心注意事项 (Precautions & Rules)

### 1. 命名策略（局部名称 vs 全路径）
* **表中的 `name` 字段必须只存当前层级的局部名称（叶子名）**，例如 `"Vocab"`，**严禁存带冒号路径的全名**（如 `"Japanese::Vocab"`）。
* 层级与全路径由 `parent_id` 树形关系动态计算。这样在重命名根牌组或移动层级时，无需大范围递归重命名所有子节点的 `name`。

### 2. 连接池调用与解引用规范
* **严禁显式解引用**：`SqlitePool` 包装器不提供 `Deref`。
* `SqlitePool` 暴露 `inner(&self) -> &Pool<Sqlite>` 方法。
* Repository/Service 统一提供 `pub fn pool(&self) -> &Pool<Sqlite> { self.pool.inner() }`，并在内部统一通过 `self.pool()` 调用，避免裸露的解引用和底层持有细节。

### 3. Repository 单元测试规范
* Repository 测试作为**单元测试**维护在当前文件的 `#[cfg(test)] mod tests` 中，与实现代码共存，提高内聚性。
* 测试中**必须统一使用 `sqlx::migrate!("../../migrations")`** 进行数据库初始化。
* **严禁在测试文件中硬编码建表 DDL**，以确保测试环境与生产迁移 100% 同步，杜绝 Schema Drift。

### 4. 动态 SQL 安全
* 涉及到变长参数（如批量删除 `DELETE ... WHERE id IN (...)`）时，**必须使用 `sqlx::QueryBuilder`** 构建动态语句并绑定参数。
* 严禁使用 `format!` 拼接 SQL 字符串，以防 SQL 注入漏洞和 SQLx 0.9+ 的编译期安全检查报错。

### 5. 时间戳字段规范
* `created_at` 与 `updated_at` 统一使用 Unix 时间戳整型（`INTEGER` / Rust `i64`），单位约定为**秒**。
