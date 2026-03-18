# User Registration

This endpoint is a browser-facing wrapper around Supabase signup. Supabase Auth remains the source of truth for account identity, and Desktop clients derive UUIDs directly from Supabase responses.

## Endpoint
- Method: POST
- Path: /api/auth/signup

## Request Body
```json
{
  "email": "user@example.com",
  "password": "secret-password",
  "username": "my-name"
}
```

## Behavior
1. Validates `email`, `password`, and `username`.
2. Calls `supabase.auth.signUp(...)` with metadata containing `username`.
3. Relies on database trigger (`on_auth_user_created`) to insert into `public.profiles` using:
  - `id`: newly created auth user id (`NEW.id`)
  - `username`: metadata username (`NEW.raw_user_meta_data->>'username'`)

## Response
```json
{
  "id": "uuid",
  "username": "my-name",
  "access_token": "... or null",
  "refresh_token": "... or null",
  "expires_at": 1234567890
}
```

## Notes
- This endpoint is designed to align with frontend `SignUpResponse`.
- Remote-first requirement: account creation must occur in Supabase first (never local-only), then local/client state is initialized using the Supabase UUID.
- Profile creation is database-owned (trigger-driven) to avoid duplicate code paths and race conditions.
- Supabase API errors (for example duplicate email or weak password) are returned by the endpoint and consumed by frontend error mapping.
- `access_token` may be `null` when email confirmation is required by project auth settings.
