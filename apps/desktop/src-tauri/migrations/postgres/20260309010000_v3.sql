-- v3 Migration: Make 'due' mandatory (NOT NULL) and ensure 'motivation' is nullable
-- This aligns the schema with the frontend Item type source of truth

-- 1. Delete items where due is NULL (cleanup before constraint)
DELETE FROM public.items WHERE due IS NULL;

-- 2. Add NOT NULL constraint to 'due' column using a constraint that can be added
ALTER TABLE public.items
    ALTER COLUMN due SET NOT NULL;

-- 3. Ensure motivation column is nullable (already is, but being explicit)
-- Note: motivation is already nullable in the schema, this is just for documentation
-- ALTER TABLE public.items
--     ALTER COLUMN motivation DROP NOT NULL;
