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
- Repository Pattern for `items`, `auth`, and `debug` operations.
- Runtime API mode switching via `src/api/config.ts` with memoized mode detection.
- Session token race protection in `useItems.ts` using `currentLoadToken`.
- Pinia-safe token injection into `HonoClient` via deferred token getter from `main.ts`.
- Strong alignment with backend contract fields (`snake_case` item properties like `duration_minutes`, `sync_status`).

## Data And Control Flow
### Authentication flow
1. `App.vue` mounts and calls `initialize()` from `useAuth()`.
2. `useAuth()` delegates to `useUserStore()`.
3. `user` store calls `authRepository.autoLogin()` and fallback `getActiveSession()`.
4. On success, store holds `userId`, `username`, and `accessToken`.

### Item loading flow with race safety
1. Login state turns true.
2. `App.vue` triggers `startNewSession()` in `useItems()`.
3. Fetches pass session token into `fetchActiveItems(token)`.
4. `useItems()` applies results only if token still matches `currentLoadToken`.
5. Logout calls `invalidateSession()`, increments token, and clears in-memory items.

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
│   │   ├── debugRepository.ts
│   │   ├── honoClient.ts
│   │   └── itemRepository.ts
│   ├── assets/
│   │   └── vue.svg
│   ├── components/
│   │   ├── DebugTools.vue
│   │   ├── SyncButton.vue
│   │   ├── SyncStatusBadge.vue
│   │   └── TaskList.vue
│   ├── composables/
│   │   ├── useAuth.ts
│   │   ├── useDebug.ts
│   │   ├── useItems.ts
│   │   └── useSyncStatus.ts
│   ├── layouts/
│   ├── stores/
│   │   └── user.ts
│   ├── types/
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

Key Functions/Exported Members:
- Default Vue component export.
- Internal handlers: `handleLogout()` and lifecycle/watch hooks.

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

Key Functions/Exported Members:
- `ItemRepository` interface.
- `CreateItemPayload` type.
- `itemRepository` singleton.
- Internal classes: `TauriItemRepository`, `HonoItemRepository`.

### `src/api/authRepository.ts`
Description:
- Repository abstraction for authentication/session retrieval.
- Decouples store logic from transport details.
- Supports both command invoke and HTTP endpoints with same API contract.

Key Functions/Exported Members:
- `AuthRepository` interface.
- `LoginResponse`, `LocalSession` types.
- `authRepository` singleton.
- `TauriAuthRepository`, `HonoAuthRepository` classes.

### `src/api/debugRepository.ts`
Description:
- Repository abstraction for dev/debug operations.
- Keeps dev tools backend-agnostic.
- Used by `useDebug` composable.
- Also provides `fetchHonoHello()` for backend connectivity testing.

Key Functions/Exported Members:
- `DebugRepository` interface.
- `debugRepository` singleton.
- `TauriDebugRepository`, `HonoDebugRepository` classes.
- `HonoHelloResponse` interface.
- Methods: `isDevMode()`, `seedDatabase()`, `resetDatabase()`, `migrateNullUserItems()`, `fetchHonoHello()`.

### `src/api/honoClient.ts`
Description:
- HTTP transport client for Hono backend.
- Handles common request logic and Authorization header injection.
- Keeps auth lookup lazy via injected token getter.

Key Functions/Exported Members:
- `HonoClient` class.
- `HonoItemsClient` interface.
- `CreateItemPayload` type.
- `honoClient` singleton.
- Request helpers: `get`, `request`, `post`, `patch`, `delete`.

### `src/assets/vue.svg`
Description:
- Static logo asset from template scaffold.
- Not coupled to business logic.

Key Functions/Exported Members:
- Static SVG file, no exports.

### `src/components/DebugTools.vue`
Description:
- Presentational debug actions panel.
- Emits events to parent instead of owning data logic.
- Keeps UI/behavior split clean with `MainDashboard.vue`.

Key Functions/Exported Members:
- Default Vue component export.
- Emits: `seed`, `reset`, `migrate`.
- Props: `visible`, `isAuthenticated`, `username`.

### `src/components/SyncButton.vue`
Description:
- Sync action control bound to auth and sync state.
- Uses composables (`useAuth`, `useItems`) for behavior.
- Displays sync error feedback with auto-clear timer.

Key Functions/Exported Members:
- Default Vue component export.
- Internal handler: `handleSync()`.

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
- Renders active task cards.
- Delegates sync-state badge display to `SyncStatusBadge`.

Key Functions/Exported Members:
- Default Vue component export.
- Props: `items`, `syncMap`, `errorMap`, `isSyncing`.

### `src/composables/useAuth.ts`
Description:
- Lightweight auth facade around `useUserStore`.
- Exposes auth state/actions to views without leaking store internals.

Key Functions/Exported Members:
- `useAuth()` returning refs/actions: `isAuthenticated`, `initialize`, `login`, `logout`, etc.

### `src/composables/useDebug.ts`
Description:
- Stateful debug workflow layer.
- Wraps `debugRepository` with local loading/error refs for UI.

Key Functions/Exported Members:
- `useDebug()`
- Actions: `checkDevMode`, `seedDatabase`, `resetDatabase`, `migrateNullUserItems`, `fetchHonoHello`.

### `src/composables/useItems.ts`
Description:
- Core item workflow and shared state module.
- Implements race-safe session token strategy.
- Delegates persistence to `itemRepository`.

Key Functions/Exported Members:
- `useItems()`
- Shared refs: `items`, `isLoading`, `isSyncing`, `error`.
- Session controls: `getCurrentToken`, `startNewSession`, `invalidateSession`.
- Actions: `fetchActiveItems`, `createItem`, `syncItems`, etc.

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

Key Functions/Exported Members:
- `useUserStore`.
- State refs: `userId`, `username`, `isInitialized`, `accessToken`.
- Actions: `initialize`, `login`, `logout`.
- Getter: `isAuthenticated`.

### `src/types/item.ts`
Description:
- Canonical item contract used across UI, composables, and repositories.
- Mirrors backend payload naming (`snake_case`).

Key Functions/Exported Members:
- `UUID` type.
- `Item` type.
- `RefreshResult` type.

### `src/views/LoginView.vue`
Description:
- Active login screen for routing.
- Uses `useAuth` for authentication command.
- Includes accessibility improvements (labels, ARIA alert).

Key Functions/Exported Members:
- Default Vue component export.
- Internal handler: `handleLogin()`.

### `src/views/MainDashboard.vue`
Description:
- Main authenticated workspace.
- Composes auth, items, debug, and sync-status workflows.
- Handles remote catch-up events and Hono hello ping.
- Includes "Add New Item" form for creating tasks with validation and loading states.

Key Functions/Exported Members:
- Default Vue component export.
- Internal actions: `loadItems`, `handleRefreshItems`, `handleCreateItem`, `seedDatabase`, `resetDatabase`, `migrateNullUserItems`, `fetchFromHono`.
- Form state: `newItemTitle`, `newItemDuration`, `newItemMotivation`, `isCreating`.

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

### Example 1: Repository usage with backend abstraction
```ts
import { itemRepository } from '@/api/itemRepository';

const id = await itemRepository.createItem({
  title: 'Portfolio task',
  motivation: 7,
  due: null,
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

## Quick Audit (Messy / Dead Code)
Current cleanup candidates identified during audit:

- `src/style.css`: scaffold defaults (`color-scheme: light dark`, centered body/app) may conflict with custom UI system.

**Recently Cleaned (March 2026):**
- ✅ Removed `src/components/HelloWorld.vue` (unused Vite starter)
- ✅ Removed `src/components/Login.vue` (legacy duplicate)
- ✅ Removed `src/api/service.ts` (migrated `fetchHonoHelloApi` to `debugRepository.ts`)

No blocking code issues remain. The codebase follows repository pattern consistently.

## Build Verification
Last verified command:

```bash
cd apps/frontend
pnpm run build
```

Result: successful TypeScript and Vite production build.
