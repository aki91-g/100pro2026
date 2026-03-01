CREATE TABLE items (
    id TEXT PRIMARY KEY NOT NULL, -- UUIDs stored as strings
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL CHECK(status IN ('backlog', 'todo', 'inprogress', 'done')),
    due TEXT, -- ISO8601 strings
    duration_minutes INTEGER,
    motivation INTEGER DEFAULT 0,
    is_archived INTEGER DEFAULT 0, -- Boolean 0/1
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    deleted_at TEXT
);