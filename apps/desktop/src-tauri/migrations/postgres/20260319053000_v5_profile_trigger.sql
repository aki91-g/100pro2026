-- v5 Migration: Sync auth.users -> public.profiles via trigger
-- This migration centralizes profile creation in Postgres for reliability.

-- Ensure helper function exists and is safe to execute by trigger context.
CREATE OR REPLACE FUNCTION public.handle_new_user()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = public
AS $$
BEGIN
  INSERT INTO public.profiles (id, username)
  VALUES (NEW.id, NEW.raw_user_meta_data->>'username')
  ON CONFLICT (id) DO UPDATE
  SET username = COALESCE(EXCLUDED.username, public.profiles.username);

  RETURN NEW;
END;
$$;

-- Ensure trigger is attached exactly once.
DROP TRIGGER IF EXISTS on_auth_user_created ON auth.users;

CREATE TRIGGER on_auth_user_created
AFTER INSERT ON auth.users
FOR EACH ROW
EXECUTE FUNCTION public.handle_new_user();
