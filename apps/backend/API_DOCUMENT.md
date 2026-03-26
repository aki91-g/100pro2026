# Backend API Document

## Overview
This service is a Hono API backed by Supabase.

UI note (2026-03):
- Frontend navigation/settings were consolidated into a header User Profile Menu.
- Theme and language are stored client-side (`localStorage`) and do not introduce new backend endpoints.
- Existing auth/item routes remain unchanged.

Base URL example:
- http://localhost:10000

Authentication:
- Most item and command routes require bearer auth.
- Header: Authorization: Bearer <supabase_access_token>

Response conventions:
- Success uses JSON payloads or HTTP 204 for no-content updates.
- Errors typically return { "error": "message" } with 4xx/5xx status.

## Health

### GET /api/hello
Description:
- Lightweight health/debug endpoint.

Response 200:
- message: string
- timestamp: ISO string

## Auth Endpoints

### POST /api/auth/login
Body:
- email: string
- password: string

Response 200:
- id: string
- username: string
- access_token: string
- refresh_token: string
- expires_at: number

Errors:
- 400 if input is missing
- 401 if Supabase authentication fails

### POST /api/auth/signup
Body:
- email: string
- password: string
- username: string

Response 200:
- id: string
- username: string
- access_token: string | null
- refresh_token: string | null
- expires_at: number | null

Errors:
- 400 for validation or Supabase signup errors

### POST /api/auth/logout
Response:
- 204 No Content

### GET /api/auth/session
Auth required: Yes

Response 200:
- id: string
- user_id: string
- username: string
- last_login: null
- is_active: 1

### POST /api/auth/auto-login
Auth required: Yes

Response 200:
- id: string
- user_id: string
- username: string
- last_login: null
- is_active: 1

## Item Endpoints

All routes below require bearer auth.

### GET /api/items/active
Response 200:
- Item[] where deleted_at is null and is_archived is false

### GET /api/items/archived
Response 200:
- Item[] where deleted_at is null and is_archived is true

### GET /api/items/deleted
Response 200:
- Item[] where deleted_at is not null

### POST /api/items
Alias:
- POST /api/items/create

Body:
- id?: string
- title: string
- description?: string | null
- motivation?: number | null
- due: string (ISO)
- durationMinutes?: number | null
- duration_minutes?: number | null

Behavior:
- status is initialized as todo
- sync_status is initialized as synced

Response 200:
- id: string

### PATCH /api/items/:id/status
Body:
- status: backlog | todo | inprogress | done

Behavior:
- Updates `status`, `updated_at`, and sets `sync_status` to `synced`

Response:
- 204 No Content

### PATCH /api/items/:id
Body:
- title: string
- description: string | null
- due: string (ISO)
- durationMinutes?: number | null
- duration_minutes?: number | null
- motivation?: number | null

Response:
- 204 No Content

### POST /api/items/:id/archive
Behavior:
- Sets `is_archived = true`, updates `updated_at`, and sets `sync_status = synced`

Response:
- 204 No Content

### POST /api/items/:id/unarchive
Behavior:
- Sets `is_archived = false`, updates `updated_at`, and sets `sync_status = synced`

Response:
- 204 No Content

### DELETE /api/items/:id
Behavior:
- Soft deletes by setting deleted_at
- Updates `updated_at` and sets `sync_status = synced`

Response:
- 204 No Content

### POST /api/items/:id/restore
Behavior:
- Restores a soft-deleted item back to active state
- Sets `deleted_at = null`, `is_archived = false`, updates `updated_at`, and sets `sync_status = synced`

Response:
- 204 No Content

### POST /api/items/sync
Response 200:
- count: number

Current behavior:
- returns count 0 (placeholder)

## Command Alias Endpoints

These mirror Tauri-style command names for compatibility. All require bearer auth.

### GET /api/commands/get_active_items
### GET /api/commands/get_archived_items
### GET /api/commands/get_deleted_items
### POST /api/commands/create_item
### POST /api/commands/update_item_status
### PATCH /api/commands/update_item/:id
### POST /api/commands/archive_item
### POST /api/commands/unarchive_item
### POST /api/commands/soft_delete_item
### POST /api/commands/restore_item
### POST /api/commands/sync_items

## Data Model

Item status values:
- backlog
- todo
- inprogress
- done

Sync status values:
- synced
- local_only
- modified
