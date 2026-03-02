CREATE TABLE items (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL CHECK(status IN ('backlog', 'todo', 'inprogress', 'done')),
    due TEXT,
    duration_minutes INTEGER,
    motivation INTEGER DEFAULT 0,
    is_archived INTEGER DEFAULT 0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    deleted_at TEXT
);

CREATE TRIGGER items_update_timestamp
AFTER UPDATE ON items
FOR EACH ROW
WHEN (NEW.updated_at IS OLD.updated_at)
BEGIN
    UPDATE items 
    SET updated_at = CURRENT_TIMESTAMP 
    WHERE id = OLD.id;
END;