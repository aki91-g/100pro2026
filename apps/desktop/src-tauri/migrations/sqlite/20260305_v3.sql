CREATE TABLE IF NOT EXISTS local_user (
  id TEXT PRIMARY KEY,          -- The UUID from Supabase
  username TEXT,
  hashed_session TEXT,          -- A local "token" to stay logged in offline
  last_login DATETIME,
  is_active INTEGER DEFAULT 1
);