-- v3 Migration: Make 'due' mandatory (NOT NULL) and ensure 'motivation' is nullable
-- This aligns the schema with the frontend Item type source of truth

-- SQLite does not support ALTER COLUMN directly for adding NOT NULL.
-- We must create a new table, copy data, drop the old one, and rename.

-- 1. Delete items where due is NULL (cleanup before constraint)
DELETE FROM items WHERE due IS NULL;

-- 2. Create new items table with corrected schema
CREATE TABLE items_new (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL CHECK(status IN ('backlog', 'todo', 'inprogress', 'done')),
    sync_status TEXT NOT NULL DEFAULT 'local_only' CHECK (sync_status IN ('synced', 'local_only', 'modified')),
    due TEXT NOT NULL,
    duration_minutes INTEGER,
    motivation INTEGER,
    is_archived BOOLEAN NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    deleted_at DATETIME
);

-- 3. Copy data from old table to new
INSERT INTO items_new 
SELECT * FROM items;

-- 4. Drop old table
DROP TABLE items;

-- 5. Rename new table
ALTER TABLE items_new RENAME TO items;

-- 6. Recreate indexes
CREATE INDEX IF NOT EXISTS idx_items_user_id ON items(user_id);

-- 7. Recreate trigger for auto-updating updated_at
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
