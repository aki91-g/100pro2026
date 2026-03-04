CREATE TABLE IF NOT EXISTS local_user (
  id TEXT PRIMARY KEY,          -- The UUID from Supabase
  username TEXT,
  hashed_session TEXT,          -- A local "token" to stay logged in offline
  last_login DATETIME,
  is_active INTEGER DEFAULT 1
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_local_user_single_active ON local_user(is_active) WHERE is_active = 1;