-- v6: Make auth signup resilient and RLS-complete for profiles.
-- Goals:
-- 1) Never fail auth signup because of profile-trigger insert issues.
-- 2) Ensure profile row exists immediately after signup.
-- 3) Allow authenticated users to read/update their own profile via RLS.

-- Normalize username extraction and keep trigger safe.
CREATE OR REPLACE FUNCTION public.handle_new_user()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = public
AS $$
DECLARE
  raw_username TEXT;
  normalized_username TEXT;
  fallback_username TEXT;
BEGIN
  raw_username := NEW.raw_user_meta_data->>'username';
  normalized_username := NULLIF(BTRIM(raw_username), '');

  IF normalized_username IS NOT NULL AND LOWER(normalized_username) = 'unknown' THEN
    normalized_username := NULL;
  END IF;

  IF normalized_username IS NOT NULL AND POSITION('@' IN normalized_username) > 0 THEN
    normalized_username := NULL;
  END IF;

  fallback_username := NULLIF(BTRIM(SPLIT_PART(COALESCE(NEW.email, ''), '@', 1)), '');
  IF fallback_username IS NOT NULL AND LOWER(fallback_username) = 'unknown' THEN
    fallback_username := NULL;
  END IF;

  -- Insert profile row immediately; avoid aborting auth.users transaction on collisions.
  BEGIN
    INSERT INTO public.profiles (id, username, registered_at)
    VALUES (
      NEW.id,
      COALESCE(normalized_username, fallback_username, 'user_' || SUBSTRING(NEW.id::TEXT, 1, 8)),
      NOW()
    )
    ON CONFLICT (id) DO UPDATE
    SET username = COALESCE(EXCLUDED.username, public.profiles.username);
  EXCEPTION
    WHEN unique_violation THEN
      INSERT INTO public.profiles (id, username, registered_at)
      VALUES (NEW.id, 'user_' || SUBSTRING(NEW.id::TEXT, 1, 8), NOW())
      ON CONFLICT (id) DO NOTHING;
    WHEN OTHERS THEN
      -- Failsafe: do not block auth signup if profile insert has an unexpected issue.
      INSERT INTO public.profiles (id, username, registered_at)
      VALUES (NEW.id, NULL, NOW())
      ON CONFLICT (id) DO NOTHING;
  END;

  RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS on_auth_user_created ON auth.users;
CREATE TRIGGER on_auth_user_created
AFTER INSERT ON auth.users
FOR EACH ROW
EXECUTE FUNCTION public.handle_new_user();

-- Backfill any missing profile rows for existing auth users.
INSERT INTO public.profiles (id, username, registered_at)
SELECT
  au.id,
  COALESCE(
    NULLIF(BTRIM(au.raw_user_meta_data->>'username'), ''),
    NULLIF(BTRIM(SPLIT_PART(COALESCE(au.email, ''), '@', 1)), ''),
    'user_' || SUBSTRING(au.id::TEXT, 1, 8)
  ) AS username,
  NOW()
FROM auth.users au
LEFT JOIN public.profiles p ON p.id = au.id
WHERE p.id IS NULL;

-- Profiles RLS policies for frontend reads/updates.
ALTER TABLE public.profiles ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS "profiles_select_own" ON public.profiles;
CREATE POLICY "profiles_select_own"
ON public.profiles
FOR SELECT
USING (auth.uid() = id);

DROP POLICY IF EXISTS "profiles_update_own" ON public.profiles;
CREATE POLICY "profiles_update_own"
ON public.profiles
FOR UPDATE
USING (auth.uid() = id)
WITH CHECK (auth.uid() = id);

DROP POLICY IF EXISTS "profiles_insert_own" ON public.profiles;
CREATE POLICY "profiles_insert_own"
ON public.profiles
FOR INSERT
WITH CHECK (auth.uid() = id);
