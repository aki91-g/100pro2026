# ItemService Integration into Repository Pattern

## ✅ Completed Integration

### 1. Type Extraction - `src/types/item.ts` (NEW)
Created a centralized types file containing:
- `UUID` type
- `Item` type (matches Rust backend)
- `RefreshResult` type (for multi-category fetches)

**Benefits:**
- Single source of truth for types
- Can be imported anywhere without circular dependencies
- Backward compatible with old imports via itemService.ts re-exports

### 2. Logic Merging - `src/api/itemRepository.ts` (UPDATED)

#### Added to ItemRepository Interface:
```typescript
refreshItems(): Promise<RefreshResult>;
syncAndRefresh(): Promise<{ count: number; data: RefreshResult }>;
```

#### TauriItemRepository Implementation:
- `refreshItems()` - Fetches all items from three categories in parallel
- `syncAndRefresh()` - Runs Rust sync then refreshes local state
- Fixed all `invoke()` calls with proper type hints and arg spreading:
  - Before: `invoke<T>('command', payload)`
  - After: `invoke<T>('command', { ...payload })`

#### HonoItemRepository Implementation:
- Same methods implemented via Hono client
- Future-ready for when Hono backend is ready

### 3. Tauri/Invoke Error Fixes - `src/api/config.ts` (UPDATED)

Fixed Tauri detection:
- Before: `if (window.__TAURI__)`
- After: `if ((window as any).__TAURI_INTERNALS__)`

### 4. Cleanup - `src/services/itemService.ts` (CONVERTED TO TYPES-ONLY)

**Before:** 70+ lines of types + functions
**After:** 11 lines (types-only + deprecation notice)

```typescript
/**
 * @deprecated This file is deprecated.
 * Use @/types/item for types.
 * Use @/api/itemRepository for logic.
 */

export type { Item, UUID, RefreshResult } from '@/types/item';
```

### 5. Updated Imports

Updated all imports across the codebase:

| File | Before | After |
|------|--------|-------|
| useItems.ts | `@/services/itemService` | `@/types/item` |
| itemRepository.ts | `@/services/itemService` | `@/types/item` |
| honoClient.ts | `@/services/itemService` | `@/types/item` |
| TaskList.vue | `@/services/itemService` | `@/types/item` |

## File Structure

```
src/
├── types/
│   └── item.ts              # ✨ NEW - Centralized types
├── api/
│   ├── config.ts            # ✓ FIXED - Tauri check
│   ├── itemRepository.ts    # ✓ UPDATED - Merged logic + fixed invoke calls
│   └── honoClient.ts        # ✓ UPDATED - Import from types
├── services/
│   ├── itemService.ts       # ✓ CONVERTED - Types-only with deprecation
│   └── apiService.ts        # Unchanged (legacy)
└── composables/
    └── useItems.ts          # ✓ UPDATED - Import from types
```

## Usage Examples

### Creating Items
```typescript
import { itemRepository } from '@/api/itemRepository';

const id = await itemRepository.createItem({
  title: 'New task',
  motivation: 5,
});
```

### Syncing and Refreshing
```typescript
const result = await itemRepository.syncAndRefresh();
console.log(`Synced ${result.count} items`);
console.log('Active items:', result.data.active);
console.log('Archived items:', result.data.archived);
console.log('Deleted items:', result.data.deleted);
```

### Using in Components
```typescript
import type { Item } from '@/types/item';

defineProps<{
  items: Item[];
}>();
```

## Backward Compatibility

The old imports still work:
```typescript
// This still works but is deprecated
import type { Item } from '@/services/itemService';
```

However, all files in the codebase have been updated to use the new structure.

## Fixed Issues

### 1. Invoke Type Errors
**Before:** `invoke<string>('create_item', payload)`
**After:** `invoke<string>('create_item', { ...payload })`

All `invoke()` calls now properly spread arguments as objects.

### 2. No-Arg Commands
**Before:** `invoke<Item[]>('get_active_items')`
**After:** `invoke<Item[]>('get_active_items', {})`

Commands without arguments now pass empty object.

### 3. Tauri Detection
**Before:** `if (window.__TAURI__) ❌` (may not exist)
**After:** `if ((window as any).__TAURI_INTERNALS__) ✅` (proper check)

## Testing Checklist

- [ ] App builds without TypeScript errors
- [ ] Item creation works
- [ ] Item fetching works
- [ ] Sync and refresh works
- [ ] Archive/delete operations work
- [ ] TaskList component renders correctly

## Next Steps

1. **Verify Build:** `npm run build` (should succeed)
2. **Test Locally:** Run dev server and test all item operations
3. **Optional:** Create tests for repository implementations
4. **Optional:** Create AuthRepository and DebugRepository following same pattern

## Migration Path

If you had custom code using old `itemService` functions:

```typescript
// OLD
import { refreshItems, syncAndRefresh } from '@/services/itemService';
const result1 = await refreshItems();
const result2 = await syncAndRefresh();

// NEW - Using repository
import { itemRepository } from '@/api/itemRepository';
const result1 = await itemRepository.refreshItems();
const result2 = await itemRepository.syncAndRefresh();
```

## Notes

- All function signatures remain the same (no breaking changes to composables)
- Types are fully backward compatible
- Ready for Hono implementation when needed
- No dependencies added
