# Local User Registration (Tauri)

Desktop registration is remote-first and online-only. Supabase account creation must complete before any local SQLite user/session writes.

## Command
- Name: register_local_user
- Location: src-tauri/src/commands/auth_commands.rs

## Parameters
- `email: string`
- `password: string`
- `username: string`

## Behavior
1. Validates required email/password/username values.
2. Calls Supabase Auth signup endpoint over HTTP (`reqwest`).
3. Supabase stores `username` in user metadata (`data.username`) and Postgres trigger writes `public.profiles`.
4. If network is unavailable or remote signup fails, returns `OFFLINE_REQUIRED_FOR_SIGNUP`.
5. Reads Supabase-returned UUID and optional access token.
6. Checks current active local user.
7. Deactivates previous active user when the id differs.
8. Upserts the new user into `local_user` with `is_active = 1`.
9. Initializes/updates singleton row in `local_session` (id = 1) using the same Supabase UUID.
10. Updates in-memory `AppState` to current user.

## SQLite Schema Alignment
- `local_user.is_active` is the active-user flag and is set to `1`.
- `local_session` stores active session context (`user_id`, `username`, optional `access_token`).

## Return
```json
{
  "id": "uuid",
  "username": "my-name"
}
```

## Notes
- Command is exposed through Tauri invoke handler and called by frontend auth repository in Tauri mode.
- Consistency guarantee: `local_user.id` and `local_session.user_id` are always the Supabase auth UUID.
- UX contract: frontend maps `OFFLINE_REQUIRED_FOR_SIGNUP` to a user-friendly online-required message.
