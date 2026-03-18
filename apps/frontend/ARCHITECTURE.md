# Frontend Architecture

## Overview
This frontend is a Vue 3 + TypeScript application using the Repository Pattern to support two backend modes without changing UI/business code:

- `tauri`: local Rust bridge (`@tauri-apps/api/core` + local SQLite flow)
- `hono`: HTTP API (Node/Hono + Supabase)

The architecture separates responsibilities into:

- API layer: backend abstraction and transport
- Composables: stateful business workflows
- Store: auth/session identity source
- Views/Components: presentation and interaction

## Core Design Decisions
- Repository Pattern for `items` and `auth` operations.
- Remote-first registration across all modes to preserve user ID consistency: Hono (`POST /api/auth/signup`) and Tauri (`register_local_user` command that signs up against Supabase first).
- Runtime API mode switching via `src/api/config.ts` with memoized mode detection.
- Session token race protection in `useItems.ts` using `currentLoadToken`.
- Pinia-safe token injection into `HonoClient` via deferred token getter from `main.ts`.
- Strong alignment with backend contract fields (`snake_case` item properties like `duration_minutes`, `sync_status`).
- **Automated 30-second sync**: Background interval timer synchronizes items automatically when authenticated, with in-flight guard to prevent concurrent syncs.
- **Schema enforcement**: `due` field is mandatory across all layers (frontend type, backend model, database schema), `motivation` is nullable, `description` is nullable.
- **ScatterPlot visualization**: Interactive SVG scatter plot (`ScatterPlot.vue`) renders task items by due date with configurable Y-axis, color, and radius fields, powered by `useGraph.ts` and D3 force simulation.
- **Debug test helpers**: `testCreate` and `testFetch` functions remain in `MainDashboard.vue` for in-development validation.
- **TaskDrawer self-contained CRUD**: `TaskDrawer.vue` calls `useItems()` directly for create, archive, and soft-delete operations, eliminating the need to delegate item mutations through parent component event handlers.

## Data And Control Flow
### Authentication flow
1. `App.vue` mounts and calls `initialize()` from `useAuth()`.
2. `useAuth()` delegates to `useUserStore()`.
3. `user` store calls `authRepository.autoLogin()` and fallback `getActiveSession()`.
4. On success, store holds `userId`, `username`, and `accessToken`.

### Registration flow
1. UI (or future signup form) calls `useAuth().signUp(email, password, username)`.
2. `useUserStore().signUp()` delegates to `authRepository.signUp()`.
3. Repository routes by API mode:
4. Hono mode calls `/api/auth/signup`.
5. Tauri mode invokes `register_local_user`, which performs Supabase signup first and then persists the returned UUID to `local_user` and `local_session`.
6. Postgres trigger (`on_auth_user_created`) creates `public.profiles` from auth metadata (`raw_user_meta_data.username`), removing app-side duplication.
7. Desktop local user switch and session write execute inside one SQLite transaction (all-or-nothing).
8. Sign-up is online-only; offline failures return `OFFLINE_REQUIRED_FOR_SIGNUP` while API failures surface descriptive server error bodies.
9. `useAuth().signUp()` maps technical signup failures into user-facing messages (existing account, weak password, unavailable service, offline).
10. Store only marks user authenticated when `access_token` is present; without it, sign-up does not hydrate authenticated state.

### Item loading flow with race safety
1. Login state turns true.
2. `App.vue` triggers `startNewSession()` in `useItems()`.
3. Fetches pass session token into `fetchActiveItems(token)`.
4. `useItems()` applies results only if token still matches `currentLoadToken`.
5. Logout calls `invalidateSession()`, increments token, and clears in-memory items.

### Automated sync flow
1. `App.vue` calls `startAutoSync()` from `useItems()` when user authenticates.
2. `useItems()` starts a 30-second interval timer that calls `syncItems()` automatically.
3. In-flight guard prevents concurrent sync operations (checks `isSyncing.value`).
4. On logout or unmount, `stopAutoSync()` clears the interval timer.
5. Manual sync via UI still works and respects same in-flight guard.

### API mode and transport flow
1. Repositories call `getApiMode()` once (memoized).
2. Factory returns Tauri or Hono implementation.
3. In Hono mode, `honoClient` attaches bearer token via injected token getter.

## API Mode Rules (`src/api/config.ts`)
Mode selection priority:
1. If Tauri runtime is detected (`window.__TAURI__` or `window.__TAURI_INTERNALS__`), use `tauri`.
2. Else, validate `VITE_API_MODE` if present.
3. Else default to `hono`.

Validation behavior:
- Allowed env values: `tauri`, `hono`
- Invalid value throws an error and logs a critical message.

Helper exports:
- `getApiMode()`
- `usesTauriBackend()`
- `usesHonoBackend()`

## File Structure
```text
apps/frontend/
├── src/
│   ├── api/
│   │   ├── authRepository.ts
│   │   ├── config.ts
│   │   ├── honoClient.ts
│   │   └── itemRepository.ts
│   ├── assets/
│   │   └── vue.svg
│   ├── components/
│   │   ├── ScatterPlot.vue
│   │   ├── SyncStatusBadge.vue
│   │   ├── TaskDrawer.vue
│   │   └── TaskList.vue
│   ├── composables/
│   │   ├── useAuth.ts
│   │   ├── useGraph.ts
│   │   ├── useItems.ts
│   │   └── useSyncStatus.ts
│   ├── layouts/
│   ├── stores/
│   │   └── user.ts
│   ├── types/
│   │   ├── graph.ts
│   │   └── item.ts
│   ├── views/
│   │   ├── LoginView.vue
│   │   └── MainDashboard.vue
│   ├── App.vue
│   ├── main.ts
│   ├── style.css
│   └── vite-env.d.ts
├── ARCHITECTURE.md
├── README.md
├── package.json
├── tsconfig.app.json
├── tsconfig.json
├── tsconfig.node.json
└── vite.config.ts
```

## File Reference (All `src` Files)

### `src/main.ts`
Description:
- Application bootstrap.
- Installs Pinia before any store access.
- Injects a deferred auth token getter into `honoClient` to avoid early store initialization errors.
- Connects bootstrap layer with API transport layer.

Key Functions/Exported Members:
- No exports.
- Calls `honoClient.setTokenGetter(() => useUserStore(pinia).accessToken ?? null)`.

### `src/App.vue`
Description:
- Root container and auth gate.
- Orchestrates initialization and auth-state transitions.
- Uses composables, not direct transport calls.
- Routes between `LoginView` and `MainDashboard`.
- Manages automated sync lifecycle: starts `autoSync` on authentication, stops on unmount.

Key Functions/Exported Members:
- Default Vue component export.
- Internal handlers: `handleLogout()` and lifecycle/watch hooks.
- Sync lifecycle: calls `startAutoSync()` when authenticated, `stopAutoSync()` on unmount.

### `src/api/config.ts`
Description:
- Centralized backend mode decision.
- Memoizes mode for performance and consistency.
- Validates env configuration for safety.

Key Functions/Exported Members:
- `getApiMode()`
- `usesTauriBackend()`
- `usesHonoBackend()`
- `ApiMode` type.

### `src/api/itemRepository.ts`
Description:
- Repository Pattern abstraction for item operations.
- Provides identical interface for Tauri and Hono backends.
- Contains composite methods (`refreshItems`, `syncAndRefresh`) used by composables.
- **Schema enforcement**: `CreateItemPayload` requires `due: string` (mandatory), `motivation: number | null` (nullable), `description?: string | null` (optional nullable).

Key Functions/Exported Members:
- `ItemRepository` interface.
- `CreateItemPayload` type with mandatory `due` field and optional `description` field.
- `itemRepository` singleton.
- Internal classes: `TauriItemRepository`, `HonoItemRepository`.

### `src/api/authRepository.ts`
Description:
- Repository abstraction for authentication/session retrieval.
- Decouples store logic from transport details.
- Supports both command invoke and HTTP endpoints with same API contract.
- Handles user registration via mode-based routing:
  - Hono mode: `POST /api/auth/signup`.
  - Tauri mode: `register_local_user` command with remote-first Supabase signup.
- Applies shared sign-up input normalization/validation before delegating to backend-specific implementations.
- Relies on database trigger for profile row creation instead of manual app-level `profiles` insert/upsert.
- Maps desktop offline sign-up failures via `OFFLINE_REQUIRED_FOR_SIGNUP` and preserves descriptive Supabase API errors.

Key Functions/Exported Members:
- `AuthRepository` interface.
- `LoginResponse`, `SignUpResponse`, `LocalSession` types.
- `authRepository` singleton.
- `TauriAuthRepository`, `HonoAuthRepository` classes.

### `src/api/honoClient.ts`
Description:
- HTTP transport client for Hono backend.
- Handles common request logic and Authorization header injection.
- Keeps auth lookup lazy via injected token getter.
- **Schema alignment**: `CreateItemPayload` matches frontend `Item` type with mandatory `due` and nullable `motivation`.

Key Functions/Exported Members:
- `HonoClient` class.
- `HonoItemsClient` interface.
- `CreateItemPayload` type with mandatory `due: string`, nullable `motivation: number | null`.
- `honoClient` singleton.
- Request helpers: `get`, `request`, `post`, `patch`, `delete`.

### `src/assets/vue.svg`
Description:
- Static logo asset from template scaffold.
- Not coupled to business logic.

Key Functions/Exported Members:
- Static SVG file, no exports.

### `src/components/ScatterPlot.vue`
Description:
- Interactive SVG scatter plot for visualizing task items by due date.
- X-axis is always due date (relative to now, in a configurable time window).
- Y-axis, color, and radius are individually selectable from item fields (`duration_minutes`, `motivation`, `status`).
- Uses `useGraph` composable for all data processing and D3 force simulation for collision-free placement.
- Supports grouping mode that aggregates nearby dots into a single marker.
- Renders triangle markers for items outside the current time window (clamped left/right).
- Refreshes layout automatically on container resize (ResizeObserver) and every 5 minutes (interval timer).
- Shows a debug stats bar (input / visible / plotted / skipped counts) and warning banners for invalid data.

Key Functions/Exported Members:
- Default Vue component export.
- Props: `items: Item[]`.
- Internal handlers: `handleGroupEnter(group)`, `handleGroupLeave()`, `updateViewportSize()`, `trianglePath(group)`, `tooltipStyle(group)`.
- Format helpers: `formatDue`, `formatMetric`, `shortId`, `formatRangeHint`, `markerTextAnchor`.
- Controls: `selectedRange`, `selectedYField`, `selectedColorField`, `selectedRadiusField`, `groupingEnabled` (bound to `useGraph` refs via `v-model`).

### `src/components/TaskDrawer.vue`
Description:
- Slide-in drawer component for all item lifecycle operations: create, view, edit, archive, and soft-delete.
- Manages four modes: `create`, `view`, `tasks`, and `edit` (internal only).
- **Self-contained CRUD**: Calls `useItems()` directly for all item mutations (`createItem`, `updateItem`, `archiveItem`, `softDeleteItem`) rather than delegating through parent event emissions.
- **Contextual Navigation**: Instead of a top navigation bar, uses context-aware buttons in the header:
  - In 'tasks' mode: Shows "+ New Task" button (top-right) to create a new task.
  - In 'view' mode: Shows "← Back to List" button (top-left) to return to the task list.
  - Close button always visible (top-right, after context-specific button if present).
- **Create mode**: Form with title, description, due datetime, duration, and motivation fields; submits via `itemRepository` through `useItems`, then emits `select-item` with the newly created item and transitions to 'view' mode. Cancel button returns to 'tasks' mode.
- **View mode**: Displays selected item details (status, due date, motivation, duration) with an "Edit" button in the top-right corner of the task card. Clicking "Edit" transitions to edit mode. Header shows "← Back to List" button to return to tasks.
- **Tasks mode**: Shows `<TaskList>` component displaying all active items. Task rows are clickable and emit `select-item` event to switch to 'view' mode for that task. Header shows "+ New Task" button in top-right to create new tasks.
- **Edit mode** (internal, not user-accessible from nav): Hidden form pre-filled with selected item data; saves via `updateItem` and includes **Archive** (amber) and **Delete** (red) action buttons with confirmation. Clicking "Cancel" returns to 'view' mode. Clicking "Save Changes" returns to 'view' mode with updated item state.
- Header title updates based on mode: "Tasks" (tasks mode), "Task Details" (view), "Create Task" (create), "Edit Task" (edit).
- Keyboard accessible: `Escape` closes the drawer.
- Mode transitions: Tasks<->Create (via "+ New Task" button), View->Tasks (via "← Back to List" button), View->Edit (via Edit button), Create->View (after creation), Tasks->View (via task click).

Key Functions/Exported Members:
- Default Vue component export.
- Props: `open`, `mode` (DrawerMode), `selectedItem`, `items`, `syncMap`, `errorMap`, `isSyncing`.
- DrawerMode type: `'create' | 'view' | 'edit' | 'tasks'`.
- Emits: `update:open`, `update:mode`, `select-item`.
- Internal navigation functions: `goToTasks()`, `goToCreate()`, `startEdit()`, `cancelEdit()`, `cancelCreate()`, `handleTaskListSelect()`.
- Internal actions: `submitCreate()`, `handleEditSubmit()`, `handleArchive()`, `handleDelete()`.
- Local loading states: `isCreating`, `isSavingEdit`, `isArchiving`, `isDeleting`.
- Navigation state: `previousMode` (preserved for potential future use).

### `src/components/SyncStatusBadge.vue`
Description:
- Pure status visualization component.
- Combines persisted `sync_status` with transient event status.

Key Functions/Exported Members:
- Default Vue component export.
- Internal helper: `getLabel()`.
- Props: `syncStatus`, `eventStatus`, `errorMessage`, `isSyncing`.

### `src/components/TaskList.vue`
Description:
- Renders active task item cards with sync status and metadata.
- Displayed within the "Tasks" tab of the drawer for quick-access viewing of all active items.
- Each task row is clickable and emits a `select-item` event to allow switching to the 'view' mode for that task.
- Delegates sync-state badge display to `SyncStatusBadge`.
- Maintains status pills (TODO, IN_PROGRESS, DONE, BACKLOG) color-coded by status.

Key Functions/Exported Members:
- Default Vue component export.
- Props: `items`, `syncMap`, `errorMap`, `isSyncing`.
- Emits: `select-item` with selected `Item` as payload.
- Styling: Card-based layout with flex alignment for status badge, status pill, task info, and motivation indicator.

### `src/composables/useAuth.ts`
Description:
- Lightweight auth facade around `useUserStore`.
- Exposes auth state/actions to views without leaking store internals.
- Maps signup errors (`OFFLINE_REQUIRED_FOR_SIGNUP`, config issues, Supabase API errors, malformed responses) to user-friendly UI text for registration UX.

Key Functions/Exported Members:
- `useAuth()` returning refs/actions: `isAuthenticated`, `initialize`, `signUp`, `login`, `logout`, etc.
- `signUp` wrapper translates technical signup errors into actionable copy for `LoginView`.

### `src/composables/useGraph.ts`
Description:
- All graph data processing logic, isolated from rendering.
- Accepts a `GraphConfig` at construction time for layout constants and palette.
- Converts raw `Item[]` into positioned `GraphItem[]` using D3 `scaleLinear`, `scaleQuantize`, and a force simulation (`forceX`, `forceY`, `forceCollide`) run synchronously for 90 ticks.
- Builds `GraphGroup[]` either as 1-to-1 wrappers (grouping off) or spatially bucketed aggregates (grouping on).
- Provides reactive `selectedRange`, `selectedYField`, `selectedColorField`, `selectedRadiusField`, `groupingEnabled` controls.
- Exposes `setDimensions` (called on resize), `updateData` (called when items change), `refreshNow` (called on control change), and `destroy` (cleanup).
- Exports `GraphDebugStats` for the debug stats bar in `ScatterPlot.vue`.

Key Functions/Exported Members:
- `useGraph(config: GraphConfig)` factory function.
- Returned refs: `graphGroups`, `warnings`, `debugStats`, `layout`, `xTicks`, `yTicks`, `selectedRange`, `selectedYField`, `selectedColorField`, `selectedRadiusField`, `groupingEnabled`.
- Returned actions: `setDimensions`, `updateData`, `refreshNow`, `destroy`.
- `GraphDebugStats` type export.

### `src/composables/useItems.ts`
Description:
- Core item workflow and shared state module.
- Implements race-safe session token strategy.
- Delegates persistence to `itemRepository`.
- **Automated sync**: Manages 30-second interval timer with `startAutoSync()`/`stopAutoSync()` and in-flight guard.
- **Schema enforcement**: `createItem()` requires `due` parameter and accepts optional `description`.

Key Functions/Exported Members:
- `useItems()`
- Shared refs: `items`, `isLoading`, `isSyncing`, `error`.
- Session controls: `getCurrentToken`, `startNewSession`, `invalidateSession`.
- Sync controls: `startAutoSync`, `stopAutoSync`.
- Actions: `fetchActiveItems`, `fetchArchivedItems`, `fetchDeletedItems`, `createItem({ title, description, motivation, due, durationMinutes })`, `syncItems`, `syncAndRefresh`, `updateItemStatus`, `archiveItem`, `softDeleteItem`.

### `src/composables/useSyncStatus.ts`
Description:
- Listens to Tauri sync-status events and stores transient per-item state.
- Manages listener lifecycle safely across mount/unmount.

Key Functions/Exported Members:
- `useSyncStatus()`
- Returned refs: `syncMap`, `errorMap`.

### `src/stores/user.ts`
Description:
- Pinia store for authenticated user/session identity.
- Owns `accessToken` used by Hono requests.
- Uses `authRepository` (not direct transport).
- Handles post-registration state hydration (`userId`, `username`, `accessToken`) in addition to login/auto-login.
- Guards authentication state on sign-up: requires a valid `access_token` before persisting authenticated identity.

Key Functions/Exported Members:
- `useUserStore`.
- State refs: `userId`, `username`, `isInitialized`, `accessToken`.
- Actions: `initialize`, `signUp`, `login`, `logout`.
- Getter: `isAuthenticated`.

### `src/types/graph.ts`
Description:
- Type definitions for the scatter-plot graph layer.
- Separates graph-specific concerns from the core `Item` type.

Key Functions/Exported Members:
- `GraphTimeRangeKey` type: `'1d' | '3d' | '1w' | '2w' | '1m'`.
- `GraphAxisField` type: `'duration_minutes' | 'motivation' | 'status'`.
- `GraphVisualField` type: `'none' | 'duration_minutes' | 'motivation' | 'status'`.
- `GraphMarker` type: `'circle' | 'triangle-left' | 'triangle-right'`.
- `Point`, `GraphTick`, `GraphItem`, `GraphGroup`, `GraphConfig`, `GraphLayout` interfaces.

### `src/types/item.ts`
Description:
- **Source of truth**: Canonical item contract used across UI, composables, and repositories.
- Mirrors backend payload naming (`snake_case`).
- **Schema enforcement**: `due: string` is mandatory (no optional/undefined), `motivation: number | null` is explicitly nullable, `description: string | null` is nullable.

Key Functions/Exported Members:
- `UUID` type.
- `Item` type with mandatory `due: string`, nullable `motivation: number | null`, and nullable `description: string | null`.
- `RefreshResult` type.

### `src/views/LoginView.vue`
Description:
- Active login screen for routing.
- Uses `useAuth` for authentication command.
- Supports dual-mode auth UI: Login and Sign Up (`isRegisterMode`).
- Sign Up mode includes username input with basic validation.
- Enforces online-only account creation using `navigator.onLine` state.
- Shows disclaimer: "An active internet connection is required to create a new account."
- After successful signup, attempts login and relies on auth gate to transition to `MainDashboard`.
- Includes accessibility improvements (labels, ARIA alert).

Key Functions/Exported Members:
- Default Vue component export.
- Internal handlers: `handleLogin()`, `handleRegister()`, `setMode()`, online status listeners.

### `src/views/MainDashboard.vue`
Description:
- Main authenticated workspace for task visualization and management.
- Composes auth, items, and sync-status workflows.
- Handles remote catch-up events via Tauri event listener.
- **ScatterPlot integration**: Renders `<ScatterPlot :items="items" />` with configurable Y-axis, color, and radius fields, and relays `select-item` clicks to the `TaskDrawer` in 'view' mode.
- **TaskDrawer orchestration**: Opens `TaskDrawer` in `create`, `view`, `tasks`, or `edit` mode; passes shared item state as props; all item mutations are handled internally by `TaskDrawer` via `useItems()`.
  - "New Task" button opens drawer in 'create' mode.
  - "Tasks" button opens drawer in 'tasks' mode (shows all active tasks for quick access).
  - Clicking a task in the scatter plot calls `handleSelectItem()` which opens drawer in 'view' mode for that task.
- **Debug helpers**: `testCreate` (creates a random debug task) and `testFetch` (logs items to console) are present for in-development use, exposed as fixed-position buttons in the UI.

Key Functions/Exported Members:
- Default Vue component export.
- Internal actions: `loadItems`, `handleRefreshItems`, `handleSelectItem`, `handleLogout`, `openDrawer`.
- Debug actions: `testCreate`, `testFetch`.

### `src/style.css`
Description:
- Global stylesheet from initial Vite scaffold.
- Provides baseline global styles and theme behavior.
- Contains defaults that may conflict with app-specific look and feel.

Key Functions/Exported Members:
- CSS-only file, no exports.

### `src/vite-env.d.ts`
Description:
- Type declarations for Vite environment typing.

Key Functions/Exported Members:
- Triple-slash reference to `vite/client` types.

## Root-Level File Reference

### `ARCHITECTURE.md`
Description:
- Canonical frontend architecture specification.
- Documents layering, responsibilities, and current implementation contracts.

Key Functions/Exported Members:
- Markdown document, no code exports.

### `README.md`
Description:
- Entry-level project guide (setup, usage, and developer onboarding).
- Complements architecture details with quick-start context.

Key Functions/Exported Members:
- Markdown document, no code exports.

### `package.json`
Description:
- Frontend package metadata, scripts, and dependency declarations.
- Controls `dev`/`build` workflows used by Vite and Vue TypeScript tooling.

Key Functions/Exported Members:
- NPM scripts and dependency manifest.

### `tsconfig.app.json`
Description:
- TypeScript compiler settings for browser app source files.
- Works with Vue tooling for strict typing during builds.

Key Functions/Exported Members:
- TypeScript config JSON, no runtime exports.

### `tsconfig.node.json`
Description:
- TypeScript settings for Node-context files (such as Vite config).

Key Functions/Exported Members:
- TypeScript config JSON, no runtime exports.

### `tsconfig.json`
Description:
- Root TypeScript project references/base config for frontend workspace.

Key Functions/Exported Members:
- TypeScript config JSON, no runtime exports.

### `vite.config.ts`
Description:
- Vite build/dev-server configuration for the frontend app.
- Defines tooling behavior, alias resolution, and bundling pipeline.

Key Functions/Exported Members:
- Default Vite config export.

## Usage Examples (Current)

### Example 1: Repository usage with schema enforcement
```ts
import { itemRepository } from '@/api/itemRepository';

// `due` is mandatory, `description` and `durationMinutes` are optional nullable
const id = await itemRepository.createItem({
  title: 'Portfolio task',
  description: 'Review chart feedback',  // optional - can be null or omitted
  motivation: 7,  // nullable - can be null
  due: '2024-03-15T10:00:00Z',  // required - must provide valid ISO string
  durationMinutes: 45,
});

const active = await itemRepository.getActiveItems();
const result = await itemRepository.syncAndRefresh();
console.log(result.count, result.data.active.length, id, active.length);
```

### Example 2: Session-safe item loading
```ts
import { useItems } from '@/composables/useItems';

const { startNewSession, fetchActiveItems, getCurrentToken, invalidateSession } = useItems();

const token = startNewSession();
await fetchActiveItems(token); // Applies only if token is still current

await fetchActiveItems(getCurrentToken()); // Manual refresh
invalidateSession(); // Called on logout to prevent stale writes
```

### Example 3: API mode decisions
```ts
import { getApiMode, usesTauriBackend, usesHonoBackend } from '@/api/config';

const mode = getApiMode();
if (usesTauriBackend()) {
  console.log('Local command bridge mode', mode);
}
if (usesHonoBackend()) {
  console.log('Remote API mode', mode);
}
```

### Example 4: Hono token injection from bootstrap
```ts
import { createPinia } from 'pinia';
import { honoClient } from '@/api/honoClient';
import { useUserStore } from '@/stores/user';

const pinia = createPinia();
honoClient.setTokenGetter(() => useUserStore(pinia).accessToken ?? null);
```

### Example 5: Automated sync lifecycle
```ts
import { useItems } from '@/composables/useItems';

// In App.vue lifecycle
const { startAutoSync, stopAutoSync } = useItems();

// Start sync on authentication
watch(isAuthenticated, (authenticated) => {
  if (authenticated) {
    const sessionToken = startNewSession();
    startAutoSync(sessionToken); // Starts 30-second interval timer
  }
});

// Stop sync on unmount
onUnmounted(() => {
  stopAutoSync(); // Clears interval timer
});
```

### Example 6: ScatterPlot usage
```vue
<!-- Inside MainDashboard.vue template -->
<ScatterPlot :items="items" />
```
```ts
// ScatterPlot receives Item[] and uses useGraph internally
// No additional wiring required — useGraph handles D3 layout
```

## Build Verification
Last verified command:

```bash
cd apps/frontend
pnpm run build
```

Result: build count increases with added graph/visualization modules (run `pnpm run build` to get latest).

## Schema Alignment Summary

### Database Schema (PostgreSQL & SQLite v3 migrations)
- `due`: `TIMESTAMPTZ NOT NULL` (PostgreSQL) / `TEXT NOT NULL` (SQLite)
- `motivation`: `INTEGER` (nullable, no constraint)

### Backend Rust (Tauri)
- `Item` model: `due: DateTime<Utc>` (required), `motivation: Option<i32>` (nullable)
- All repository traits and implementations aligned with mandatory `due` and nullable `motivation`

### Backend Hono (Node.js)
- `ItemRow` type: `due: string` (required), `motivation: number | null` (nullable)
- `normalizeIso()` handles nullable timestamps correctly

### Frontend (Vue 3 + TypeScript)
- **Item type** (source of truth): `due: string` (required), `motivation: number | null` (nullable), `description: string | null` (nullable)
- `CreateItemPayload` enforces mandatory `due` field and accepts optional `description`
- Form validation requires `due` input before item creation
- Automated 30-second sync with in-flight guard

### Migration Strategy
- V3 migrations delete any existing items with NULL `due` values
- Alters `due` column to NOT NULL constraint
- Preserves nullable `motivation` column (no changes)

## Feature History

### Debug Layer Cleanup (prior)
Earlier debug infrastructure removed from the repository layer:
- **Deleted files**: `DebugTools.vue`, `SyncButton.vue`, `useDebug.ts`, `debugRepository.ts`
- **Removed commands**: `seed_database`, `reset_database`, `migrate_null_user_items` from Rust commands
- **Removed services**: `DebugService` from Rust backend

### Current Debug State
`MainDashboard.vue` retains development-only helpers:
- **`testCreate`**: Creates a randomly-titled task with a random motivation score, logs result to console.
- **`testFetch`**: Fetches active items and logs the full array to console.
- These are exposed as fixed-position buttons in the UI. Remove before shipping to production.

### Automated Sync Implementation
- **Interval**: 30-second automatic synchronization when authenticated
- **Guards**: In-flight protection (`autoSyncInFlight` flag) prevents concurrent sync operations
- **Lifecycle**: Auto-starts on authentication, auto-stops on logout/unmount

### ScatterPlot Visualization
- **Component**: `ScatterPlot.vue` renders all active items as an interactive SVG scatter plot.
- **Engine**: `useGraph.ts` handles scale computation, D3 force simulation, grouping, and tick generation.
- **Types**: `graph.ts` defines all graph-layer interfaces (`GraphItem`, `GraphGroup`, `GraphConfig`, `GraphLayout`, etc.).
- **Controls**: Time window (1d–1m), Y-axis field, color field, radius field, and grouping mode are all user-configurable.
- **Responsiveness**: ResizeObserver drives `setDimensions` on container size changes; 5-minute refresh timer keeps relative time labels current.
