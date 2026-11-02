-- Add migration script here
CREATE TABLE IF NOT EXISTS DECKS (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  is_collapsed INTEGER NOT NULL,
  parent_id INTEGER,
  config_id INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_decks_parent_id ON DECKS (parent_id);

CREATE INDEX IF NOT EXISTS idx_decks_config_id ON DECKS (config_id);

-- Field 的 field_type 按 Serde 默认枚举名称存储：Str、Json、Media。
-- 字段定义及 content 的逐字段类型匹配由 Rust 模型负责校验。
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
