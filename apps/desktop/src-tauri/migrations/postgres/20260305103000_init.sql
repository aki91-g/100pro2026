-- 1. Profiles Table
CREATE TABLE IF NOT EXISTS public.profiles (
  id UUID REFERENCES auth.users ON DELETE CASCADE NOT NULL PRIMARY KEY,
  username TEXT UNIQUE,
  registered_at TIMESTAMPTZ DEFAULT NOW()
);

-- 2. Items Table (Consolidated)
CREATE TABLE IF NOT EXISTS public.items (
    id UUID PRIMARY KEY NOT NULL,
    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE, -- Changed to NOT NULL
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL CHECK(status IN ('backlog', 'todo', 'inprogress', 'done')),
    sync_status TEXT NOT NULL DEFAULT 'local_only' CHECK (sync_status IN ('synced', 'local_only', 'modified')),
    due TIMESTAMPTZ,
    duration_minutes INTEGER,
    motivation INTEGER DEFAULT 0,
    is_archived BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- 3. Trigger Function
CREATE OR REPLACE FUNCTION trigger_set_timestamp()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 4. Apply Trigger
DROP TRIGGER IF EXISTS set_timestamp ON items;
CREATE TRIGGER set_timestamp
BEFORE UPDATE ON items
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_timestamp();

-- 5. Indexes and RLS
CREATE INDEX IF NOT EXISTS items_user_id_idx ON items (user_id);
ALTER TABLE items ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can only access their own items"
ON items FOR ALL
USING (auth.uid() = user_id)
WITH CHECK (auth.uid() = user_id);