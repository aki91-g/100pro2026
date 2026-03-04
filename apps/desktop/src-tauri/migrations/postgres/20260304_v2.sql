ALTER TABLE items 
ADD COLUMN user_id UUID REFERENCES auth.users(id) ON DELETE CASCADE;

CREATE INDEX IF NOT EXISTS items_user_id_idx ON items (user_id);

ALTER TABLE items ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can only access their own items" 
ON items 
FOR ALL 
USING (auth.uid() = user_id)
WITH CHECK (auth.uid() = user_id);