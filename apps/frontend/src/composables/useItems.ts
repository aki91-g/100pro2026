import { ref, watch, type Ref } from 'vue';
import type { Item } from '@/types/item';
import { itemRepository } from '@/api/itemRepository';

/**
 * Shared state for items across the application
 * Ensures all components see the same items/sync status
 */
const items = ref<Item[]>([]);
const isLoading = ref(false);
const error = ref<string | null>(null);
const isSyncing = ref(false);

// Auto-sync state shared across composable instances
let autoSyncTimer: ReturnType<typeof setInterval> | null = null;
let autoSyncInFlight = false;

// Reference count for concurrent loading operations
let loadingCount = 0;

// Session token for preventing race conditions on auth state changes
let currentLoadToken = 0;
let stopSyncStatusBinding: (() => void) | null = null;
type SyncStatus = 'pending' | 'success' | 'error' | undefined;

/**
 * Composable for managing items/tasks
 * Handles fetching, creation, updates, and syncing with race condition protection
 */
export function useItems() {

  function reconcileSyncStatus(syncMap: Record<string, SyncStatus>): void {
    items.value.forEach((item) => {
      const status = syncMap[item.id];
      if (status === 'success') {
        item.sync_status = 'synced';
      } else if (status === 'pending' && item.sync_status !== 'local_only') {
        item.sync_status = 'modified';
      }
    });
  }

  function bindSyncStatusMap(syncMap: Ref<Record<string, SyncStatus>>): void {
    stopSyncStatusBinding?.();
    stopSyncStatusBinding = watch(
      syncMap,
      (nextMap) => {
        reconcileSyncStatus(nextMap);
      },
      { deep: true, immediate: true },
    );
  }

  // Request counting for concurrency-safe isLoading
  function startLoading() {
    loadingCount++;
    isLoading.value = true;
  }

  function finishLoading() {
    loadingCount = Math.max(0, loadingCount - 1);
    isLoading.value = loadingCount > 0;
  }

  /**
   * Get the current load token (for external callers like MainDashboard)
   */
  function getCurrentToken(): number {
    return currentLoadToken;
  }

  /**
   * Invalidate current session (e.g., on logout)
   */
  function invalidateSession(): void {
    currentLoadToken++;
    items.value = [];
  }

  /**
   * Start a new load session and return its token
   */
  function startNewSession(): number {
    currentLoadToken++;
    return currentLoadToken;
  }

  // Fetch active items with session token protection
  async function fetchActiveItems(sessionToken?: number): Promise<Item[]> {
    startLoading();
    error.value = null;
    try {
      const data = await itemRepository.getActiveItems();
      
      // Only update if this is the current session
      if (sessionToken === undefined || sessionToken === currentLoadToken) {
        items.value = data;
      }
      
      return data;
    } catch (err) {
      // Only update error if this is the current session
      if (sessionToken === undefined || sessionToken === currentLoadToken) {
        error.value = String(err);
        console.error('Failed to fetch active items:', err);
      }
      throw err;
    } finally {
      finishLoading();
    }
  }

  // Fetch archived items
  async function fetchArchivedItems(): Promise<Item[]> {
    startLoading();
    error.value = null;
    try {
      const data = await itemRepository.getArchivedItems();
      return data;
    } catch (err) {
      error.value = String(err);
      console.error('Failed to fetch archived items:', err);
      throw err;
    } finally {
      finishLoading();
    }
  }

  // Fetch deleted items
  async function fetchDeletedItems(): Promise<Item[]> {
    startLoading();
    error.value = null;
    try {
      const data = await itemRepository.getDeletedItems();
      return data;
    } catch (err) {
      error.value = String(err);
      console.error('Failed to fetch deleted items:', err);
      throw err;
    } finally {
      finishLoading();
    }
  }

  // Sync with remote
  async function syncItems(): Promise<number> {
    isSyncing.value = true;
    error.value = null;
    try {
      const syncedCount = await itemRepository.syncItems();
      // Refresh local items after sync
      await fetchActiveItems();
      return syncedCount;
    } catch (err) {
      error.value = String(err);
      console.error('Sync failed:', err);
      throw err;
    } finally {
      isSyncing.value = false;
    }
  }

  async function syncAndRefresh(_sessionToken?: number): Promise<number> {
    if (autoSyncInFlight) {
      return 0;
    }

    autoSyncInFlight = true;
    try {
      const count = await syncItems();
      return count;
    } finally {
      autoSyncInFlight = false;
    }
  }

  function startAutoSync(sessionToken?: number, intervalMs: number = 30000): void {
    if (autoSyncTimer) {
      return;
    }

    autoSyncTimer = setInterval(() => {
      void syncAndRefresh(sessionToken).catch((err) => {
        console.error('Auto-sync failed:', err);
      });
    }, intervalMs);
  }

  function stopAutoSync(): void {
    if (autoSyncTimer) {
      clearInterval(autoSyncTimer);
      autoSyncTimer = null;
    }
  }

  // Create a new item
  async function createItem(payload: {
    title: string,
    description: string | null,
    motivation: number | null,
    due: string,
    durationMinutes?: number | null
  }): Promise<string> {
    error.value = null;
  try {
    const id = await itemRepository.createItem(payload);
    await fetchActiveItems();
    return id;
    } catch (err) {
      error.value = String(err);
      console.error('Failed to create item:', err);
      throw err;
    }
  }

  // Update item details
  async function updateItem(payload: {
    id: string,
    title: string,
    description: string | null,
    motivation: number | null,
    due: string,
    durationMinutes?: number | null
  }): Promise<void> {
    error.value = null;
    try {
      await itemRepository.updateItem(payload);
      await fetchActiveItems();
    } catch (err) {
      error.value = String(err);
      console.error('Failed to update item:', err);
      throw err;
    }
  }

  // Update item status
  async function updateItemStatus(id: string, status: Item['status']): Promise<void> {
    error.value = null;
    try {
      await itemRepository.updateItemStatus(id, status);
      // Update local state
      const item = items.value.find((i) => i.id === id);
      if (item) {
        item.status = status;
        item.sync_status = item.sync_status === 'local_only' ? 'local_only' : 'modified';
      }
    } catch (err) {
      error.value = String(err);
      console.error('Failed to update item status:', err);
      throw err;
    }
  }

  // Archive item
  async function archiveItem(id: string): Promise<void> {
    error.value = null;
    try {
      await itemRepository.archiveItem(id);
      // Remove from local list
      items.value = items.value.filter((i) => i.id !== id);
    } catch (err) {
      error.value = String(err);
      console.error('Failed to archive item:', err);
      throw err;
    }
  }

  // Soft delete item
  async function softDeleteItem(id: string): Promise<void> {
    error.value = null;
    try {
      await itemRepository.softDeleteItem(id);
      // Remove from local list
      items.value = items.value.filter((i) => i.id !== id);
    } catch (err) {
      error.value = String(err);
      console.error('Failed to delete item:', err);
      throw err;
    }
  }

  return {
    // Shared State (all instances share the same refs)
    items,
    isLoading,
    error,
    isSyncing,
    // Session Management
    getCurrentToken,
    invalidateSession,
    startNewSession,
    bindSyncStatusMap,
    // Actions
    fetchActiveItems,
    fetchArchivedItems,
    fetchDeletedItems,
    syncItems,
    syncAndRefresh,
    startAutoSync,
    stopAutoSync,
    createItem,
    updateItem,
    updateItemStatus,
    archiveItem,
    softDeleteItem,
  };
}
