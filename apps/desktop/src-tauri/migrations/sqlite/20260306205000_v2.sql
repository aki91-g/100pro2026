CREATE TABLE IF NOT EXISTS local_session (
    id INTEGER PRIMARY KEY CHECK (id = 1), -- Only ever one active session
    user_id TEXT NOT NULL,
    username TEXT NOT NULL,
    access_token TEXT, -- Optional: store the Supabase JWT
    last_login DATETIME DEFAULT CURRENT_TIMESTAMP
);