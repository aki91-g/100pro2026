ALTER TABLE items ADD COLUMN user_id TEXT;

CREATE INDEX IF NOT EXISTS idx_items_user_id ON items(user_id);