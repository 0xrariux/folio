CREATE TABLE IF NOT EXISTS preferences (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS page_history (id TEXT PRIMARY KEY, page_id TEXT NOT NULL, created_at INTEGER NOT NULL, content_json TEXT NOT NULL);
CREATE INDEX IF NOT EXISTS history_page ON page_history(page_id, created_at DESC);
