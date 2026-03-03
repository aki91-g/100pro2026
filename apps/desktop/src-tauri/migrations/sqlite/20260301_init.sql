CREATE TABLE items (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL CHECK(status IN ('backlog', 'todo', 'inprogress', 'done')),
    due TEXT,
    duration_minutes INTEGER,
    motivation INTEGER DEFAULT 0,
    is_archived BOOLEAN NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    deleted_at DATETIME
);

CREATE TRIGGER items_update_timestamp
AFTER UPDATE ON items
FOR EACH ROW
WHEN (NEW.updated_at IS OLD.updated_at)
BEGIN
    UPDATE items 
    SET updated_at = (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')) 
    WHERE id = OLD.id;
END;