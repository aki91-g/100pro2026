# Desktop Tauri API Document

## Overview
Desktop exposes Rust commands via Tauri invoke.

Frontend command call style:
- invoke("command_name", payload)

Most item commands require an authenticated user in AppState.

## Session And Global Commands

### sync_items
Signature:
- sync_items() -> Result<usize, String>

Description:
- Synchronizes local and remote items for current user.

### set_user
Signature:
- set_user(user_id: Uuid) -> Result<(), String>

### get_current_user
Signature:
- get_current_user() -> Result<Option<String>, String>

### clear_user
Signature:
- clear_user() -> Result<(), String>

## Auth Commands

### login
Signature:
- login(email: String, password: String) -> Result<LoginResponse, String>

Response:
- id: string
- username: string

Behavior:
- Authenticates against Supabase
- Resolves username from profiles
- Persists local_user and local_session
- Updates AppState

### register_local_user
Signature:
- register_local_user(email: String, password: String, username: String) -> Result<SignupResponse, String>

Response:
- id: string
- username: string
- access_token: string | null

Behavior:
- Remote-first signup in Supabase
- Persists local_user transactionally
- Creates local_session only when access_token is present
- Updates AppState only when access_token is present

### logout
Signature:
- logout() -> Result<(), String>

Behavior:
- Clears AppState and local session

### get_active_session
Signature:
- get_active_session() -> Result<Option<SafeSession>, String>

SafeSession:
- user_id: string
- username: string
- last_login: string | null

### auto_login
Signature:
- auto_login() -> Result<Option<SafeSession>, String>

Behavior:
- Restores active session from local_session table if present

## Item Commands

### get_active_items
Signature:
- get_active_items() -> Result<Vec<Item>, String>

### get_archived_items
Signature:
- get_archived_items() -> Result<Vec<Item>, String>

### get_deleted_items
Signature:
- get_deleted_items() -> Result<Vec<Item>, String>

### create_item
Signature:
- create_item(title: String, description: Option<String>, motivation: Option<i32>, due: DateTime<Utc>, duration_minutes: Option<i32>) -> Result<Uuid, String>

### update_item_status
Signature:
- update_item_status(id: Uuid, status: TaskStatus) -> Result<(), String>

TaskStatus values:
- backlog
- todo
- inprogress
- done

### update_item_details
Signature:
- update_item_details(id: Uuid, title: String, description: Option<String>, due: DateTime<Utc>, duration_minutes: Option<i32>, motivation: Option<i32>) -> Result<(), String>

### archive_item
Signature:
- archive_item(id: Uuid) -> Result<(), String>

### unarchive_item
Signature:
- unarchive_item(id: Uuid) -> Result<(), String>

### soft_delete_item
Signature:
- soft_delete_item(id: Uuid) -> Result<(), String>

### restore_item
Signature:
- restore_item(id: Uuid) -> Result<(), String>

### hard_delete_item
Signature:
- hard_delete_item(id: Uuid) -> Result<(), String>

### empty_item_trash
Signature:
- empty_item_trash() -> Result<(), String>

## Error Shape
Commands return Result<..., String>.

Frontend should handle failures by reading the error string and showing user-friendly messages where needed.
