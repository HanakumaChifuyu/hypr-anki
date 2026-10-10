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
