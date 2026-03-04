CREATE TABLE public.profiles (
  id UUID REFERENCES auth.users NOT NULL PRIMARY KEY,
  username TEXT UNIQUE,
  registered_at TIMESTAMPTZ DEFAULT NOW()
);