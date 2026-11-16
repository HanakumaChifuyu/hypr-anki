ALTER TABLE CARDS ADD COLUMN last_reviewed_at INTEGER;
ALTER TABLE CARDS ADD COLUMN due_at INTEGER;
CREATE INDEX idx_cards_due_at ON CARDS(due_at);

CREATE TABLE CARD_REVIEWS (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    card_id INTEGER NOT NULL REFERENCES CARDS(id) ON DELETE CASCADE,
    reviewed_at INTEGER NOT NULL,
    rating INTEGER NOT NULL CHECK (rating IN (1, 3)),
    elapsed_seconds INTEGER NOT NULL CHECK (elapsed_seconds >= 0),
    scheduled_seconds INTEGER NOT NULL CHECK (scheduled_seconds > 0),
    memory_before TEXT,
    memory_after TEXT NOT NULL,
    algorithm TEXT NOT NULL,
    parameters TEXT NOT NULL,
    desired_retention REAL NOT NULL
) STRICT;
CREATE INDEX idx_card_reviews_card_id ON CARD_REVIEWS(card_id, id);
