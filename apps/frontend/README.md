# Frontend (Vue + TypeScript)

TaskGraph frontend for the 100pro2026 workspace.

## Tech Stack
- Vue 3 + TypeScript + Vite
- Pinia for session/auth state
- Repository pattern for backend abstraction
- Dual backend modes: `tauri` and `hono`

## Run
```bash
npm install
npm run dev
```

## Build
```bash
npm run build
```

## API Modes
Backend mode is selected in `src/api/config.ts`:
- `tauri`: auto-selected when running inside Tauri runtime
- `hono`: default for browser environments

Optional env vars:
- `VITE_API_MODE=tauri|hono`
- `VITE_HONO_BASE_URL=<url>`

`honoClient` base URL resolution order:
1. Constructor override
2. `VITE_HONO_BASE_URL`
3. `http://localhost:10000` when running on localhost/127.0.0.1
4. Render fallback URL

## Item Mutation Paths (Canonical)
Drawer actions use canonical APIs through `useItems()` -> `itemRepository` -> `honoClient`:
- Update details: `PATCH /api/items/:id`
- Update status: `PATCH /api/items/:id/status`
- Archive: `POST /api/items/:id/archive`
- Delete (soft delete): `DELETE /api/items/:id`

Legacy alias routes remain server-side for backward compatibility, but frontend calls canonical `:id` routes.

## Key Files
- `src/components/TaskDrawer.vue`: create/view/edit drawer and task actions
- `src/components/TaskList.vue`: task list with typed `select-item` and `edit-item` events
- `src/composables/useItems.ts`: shared item state and item action APIs
- `src/api/itemRepository.ts`: backend-agnostic item repository
- `src/api/honoClient.ts`: canonical HTTP transport client
