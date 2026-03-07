# Frontend Architecture Refactoring

## Overview
This document describes the refactored frontend architecture following the Repository Pattern for clean separation of concerns.

## Architecture Changes

### 1. Repository Pattern Implementation

**New Directory: `src/api/`**

#### `config.ts`
- Determines which backend to use (Tauri local or Hono remote)
- Provides `getApiMode()`, `isTauriMode()`, `isHonoMode()` helpers
- Currently defaults to Tauri; ready for Hono integration

#### `itemRepository.ts`
- **Abstract repository interface** for all item operations
- Implements two strategies:
  - `TauriItemRepository` - Uses Tauri's `invoke()` for local SQLite
  - `HonoItemRepository` - Uses fetch/Hono client for remote API
- Factory pattern automatically selects the right implementation
- Single source of truth for all item-related API calls

#### `honoClient.ts`
- **Placeholder** for future Hono RPC integration
- Provides RESTful endpoints as a fallback
- Ready to swap with `hc` (Hono Client) when backend is implemented
- Type-safe interface matching the repository contract

### 2. Component Reorganization

#### `src/App.vue` (Simplified)
**Before:** 250+ lines with business logic  
**After:** ~130 lines, pure container

**Responsibilities:**
- Authentication state initialization
- Login/Logout routing
- Session management coordination
- Renders `Login` or `MainDashboard` based on auth state

#### `src/views/MainDashboard.vue` (New)
**All business logic moved here:**
- Item loading with session tokens
- Remote catchup event handling
- Debug commands (seed, reset, migrate)
- Hono API testing
- Sync status watching
- UI state management

### 3. Enhanced Composables

#### `src/composables/useItems.ts` (Updated)
**New Features:**
- **Session token management** - Prevents race conditions on auth changes
- `getCurrentToken()` - Get active session token
- `startNewSession()` - Create new session on login
- `invalidateSession()` - Clear state on logout
- `fetchActiveItems(sessionToken?)` - Race-safe fetching
- Now uses `itemRepository` instead of direct API calls

**Token Flow:**
```
Login → startNewSession() → returns token
Fetch → passes token → updates only if token matches
Logout → invalidateSession() → increments token, clears items
```

## File Structure

```
src/
├── App.vue                          # Simplified entry point
├── views/
│   └── MainDashboard.vue           # Main business logic
├── composables/
│   ├── useAuth.ts                  # Auth state management
│   ├── useItems.ts                 # Item operations (enhanced)
│   └── useSyncStatus.ts            # Real-time sync tracking
├── api/                            # NEW: Repository layer
│   ├── config.ts                  # API mode selection
│   ├── itemRepository.ts          # Abstract repository
│   ├── authRepository.ts          # Auth repository
│   ├── debugRepository.ts         # Debug repository
│   └── honoClient.ts              # Hono placeholder
├── services/
│   ├── apiService.ts              # Legacy (deprecated)
│   └── itemService.ts             # Types
├── components/
│   ├── Login.vue                  # Kept for backward compatibility
│   ├── SyncButton.vue
│   ├── TaskList.vue
│   └── DebugTools.vue
└── stores/
    └── user.ts                     # Pinia user store
```

## Usage Examples

### Using the Repository in a New Component

```typescript
import { itemRepository } from '@/services/api/itemRepository';

// Create an item
const id = await itemRepository.createItem({
  title: 'New task',
  motivation: 5,
});

// Fetch items
const items = await itemRepository.getActiveItems();

// Sync
const count = await itemRepository.syncItems();
```

### Using Items Composable with Session Safety

```typescript
import { useItems } from '@/composables/useItems';

const { items, fetchActiveItems, startNewSession, getCurrentToken } = useItems();

// On login
const token = startNewSession();
await fetchActiveItems(token);

// Manual refresh
await fetchActiveItems(getCurrentToken());

// On logout (automatic in App.vue)
invalidateSession();
```

## Benefits

### 1. **Separation of Concerns**
- App.vue: Routing & auth orchestration
- MainDashboard: UI & user interactions
- Repository: API abstraction
- Composables: Shared state & business logic

### 2. **Race Condition Protection**
Session tokens prevent stale data updates after logout/login cycles.

### 3. **Future-Proof**
Easy to swap Tauri for Hono by changing `getApiMode()` config.

### 4. **Testability**
Repository pattern allows mocking API layers for unit tests.

### 5. **Type Safety**
All API contracts defined in `ItemRepository` interface.

## Migration Path to Hono

When Hono backend is ready:

1. **Update `honoClient.ts`:**
```typescript
import { hc } from 'hono/client'
import type { AppType } from '../../../backend/src/index'

const client = hc<AppType>('/api')

export class HonoClient implements HonoItemsClient {
  async getActiveItems() {
    const res = await client.items.active.$get()
    return res.json()
  }
  // ... other methods
}
```

2. **Update `config.ts`:**
```typescript
export function getApiMode(): ApiMode {
  if (import.meta.env.VITE_USE_HONO === 'true') {
    return 'hono';
  }
  return 'tauri';
}
```

3. **Set environment variable:**
```bash
VITE_USE_HONO=true
```

No changes needed in components or composables!

## Deprecation Notice

`src/services/apiService.ts` is now **deprecated**. All new code should use:
- `itemRepository` for item operations
- `useItems()` composable for reactive state
- Direct Tauri `invoke()` only for auth/debug commands (until those get their own repos)

## Next Steps

1. Implement Hono backend with matching API contracts
4. Add unit tests for repositories
5. Add E2E tests for repository switching
