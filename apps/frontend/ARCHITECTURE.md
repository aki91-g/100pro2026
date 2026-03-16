# Frontend Architecture

## Overview
The frontend is a Vue 3 + TypeScript application that supports two backend pathways behind a shared API layer:
- `tauri` mode: Tauri invoke bridge + local SQLite workflow
- `hono` mode: HTTP API + Supabase-backed server

UI code does not branch on backend-specific details; it uses composables and repositories.

## Layering
- `src/components/*`: presentation and interaction
- `src/composables/*`: shared state + workflow orchestration
- `src/api/*`: transport/repository abstraction
- `src/stores/*`: auth/session state
- `src/types/*`: canonical contracts

## Item API Architecture
### Canonical frontend mutation contract
All item mutations are routed through:
1. Component (`TaskDrawer.vue`)
2. `useItems()`
3. `itemRepository`
4. Backend-specific implementation (`invoke` for Tauri, `honoClient` for Hono)

### Canonical Hono endpoints used by frontend
- `POST /api/items`
- `PATCH /api/items/:id`
- `PATCH /api/items/:id/status`
- `POST /api/items/:id/archive`
- `DELETE /api/items/:id`
- `GET /api/items/active`
- `GET /api/items/archived`
- `GET /api/items/deleted`

Server aliases still exist for compatibility, but frontend uses the canonical `:id` routes.

## Drawer Flow (`TaskDrawer.vue`)
Modes:
- `create`: form submit creates item via `createItem`
- `view`: selected task card + task list
- `edit`: full-drawer edit form with save/archive/delete

Mutation methods used by drawer:
- `createItem(payload)`
- `updateItem(payload)`
- `archiveItem(id)`
- `deleteItem(id)`

`Item` typing is preserved end-to-end (props, emits, composables, repository payloads).

## Type Safety Notes
- `TaskDrawer` emits `select-item` with `Item`
- `TaskList` emits `select-item` and `edit-item` with `Item`
- `UpdateItemPayload` and `CreateItemPayload` are explicit interfaces
- `due` is required and normalized as ISO string
- nullable fields remain typed (`description`, `motivation`, `durationMinutes`)

## API Mode Resolution
`src/api/config.ts` selects mode using:
1. Tauri runtime detection (`window.__TAURI__` / `window.__TAURI_INTERNALS__`)
2. `VITE_API_MODE` override (outside Tauri)
3. default to `hono`

## Connectivity Strategy
`honoClient` URL selection supports both local and deployed environments:
1. explicit constructor value
2. `VITE_HONO_BASE_URL`
3. localhost fallback (`http://localhost:10000`) when app host is localhost/127.0.0.1
4. Render fallback URL

Backend CORS allows localhost origins by default and merges configured origins from `CORS_ALLOWED_ORIGINS`.
