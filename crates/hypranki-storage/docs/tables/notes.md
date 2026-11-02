# 笔记类型与笔记

`NOTETYPES` 保存字段定义，`NOTES` 通过 `note_type` 引用类型。

| 表 | 字段 | SQLite 类型 | 约束与含义 |
| --- | --- | --- | --- |
| NOTETYPES | id | INTEGER | 自增主键 |
| NOTETYPES | name | TEXT | 非空类型名称 |
| NOTETYPES | fields | TEXT | 非空 JSON 数组，序列化的 `Vec<Field>` |
| NOTES | id | INTEGER | 自增主键 |
| NOTES | note_type | INTEGER | 非空外键，引用 NOTETYPES.id，并建立索引 |
| NOTES | content | TEXT | 非空 JSON 对象，序列化的 `serde_json::Value` |

除自增主键外，各列无默认值。两张表均使用 STRICT 模式。
`FieldType` 按 Serde 默认表示存为 `Str`、`Json`、`Media`。
存在引用笔记时，删除笔记类型会返回数据库外键错误。

```sql
CREATE TABLE IF NOT EXISTS NOTETYPES (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  fields TEXT NOT NULL CHECK (
    CASE WHEN json_valid(fields) THEN json_type(fields) = 'array' ELSE 0 END
  )
) STRICT;

CREATE TABLE IF NOT EXISTS NOTES (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  note_type INTEGER NOT NULL REFERENCES NOTETYPES (id) ON DELETE RESTRICT,
  content TEXT NOT NULL CHECK (
    CASE WHEN json_valid(content) THEN json_type(content) = 'object' ELSE 0 END
  )
) STRICT;

CREATE INDEX IF NOT EXISTS idx_notes_note_type ON NOTES (note_type);
```

两个仓储均提供 `create`、`find_by_id`、`find_all`、`update`、`delete_by_id`。
创建分别接收 `CreateNoteType`、`CreateNote`；更新接收完整实体。
查询不存在的 ID 返回 `None`，更新或删除不存在的 ID 返回 `false`。
NoteRepository 额外提供 `find_by_note_type`，列表按 ID 升序返回。
JSON 通过 `sqlx::types::Json` 绑定，读取时还原为模型类型。
仓储只负责持久化，不自动调用 `Note::validate`；字段业务校验由调用方负责。
修改笔记类型字段定义不会自动改写现有笔记。
