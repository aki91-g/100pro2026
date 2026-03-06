-- 1. Items Table (Consolidated)
CREATE TABLE IF NOT EXISTS items (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL, -- Changed to NOT NULL
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL CHECK(status IN ('backlog', 'todo', 'inprogress', 'done')),
    sync_status TEXT NOT NULL DEFAULT 'local_only' CHECK (sync_status IN ('synced', 'local_only', 'modified')),
    due TEXT,
    duration_minutes INTEGER,
    motivation INTEGER DEFAULT 0,
    is_archived BOOLEAN NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    deleted_at DATETIME
);

-- 2. Local User Table
CREATE TABLE IF NOT EXISTS local_user (
  id TEXT PRIMARY KEY NOT NULL,
  username TEXT,
  hashed_session TEXT,
  last_login DATETIME,
  is_active INTEGER DEFAULT 1
);

-- 3. Indexes
CREATE INDEX IF NOT EXISTS idx_items_user_id ON items(user_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_local_user_single_active ON local_user(is_active) WHERE is_active = 1;

-- 4. Trigger for Auto-Updating updated_at
DROP TRIGGER IF EXISTS items_update_timestamp;
CREATE TRIGGER items_update_timestamp
AFTER UPDATE ON items
FOR EACH ROW
WHEN (NEW.updated_at IS OLD.updated_at)
BEGIN
    UPDATE items 
    SET updated_at = (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')) 
    WHERE id = OLD.id;
END;