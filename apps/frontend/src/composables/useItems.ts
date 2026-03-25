import { ref, watch, type Ref } from 'vue';
import type { Item } from '@/types/item';
import { itemRepository } from '@/api/itemRepository';
import { useAuth } from '@/composables/useAuth';

/**
 * Shared state for items across the application
 * Ensures all components see the same items/sync status
 */
const items = ref<Item[]>([]);
const isLoading = ref(false);
const error = ref<string | null>(null);
const isSyncing = ref(false);

/**
 * Generate sample seed data for guest mode to showcase the application's capabilities
 */
function generateGuestSeedItems(userId: string): Item[] {
  const now = new Date();
  const day = 24 * 60 * 60 * 1000;
  
  const sampleItems: Item[] = [
    {
      id: `guest_sample_1_${Date.now()}`,
      user_id: userId,
      sync_status: 'local_only',
      title: 'Design new landing page',
      description: 'Create mockups and get stakeholder feedback',
      status: 'inprogress',
      due: new Date(now.getTime() + 2 * day).toISOString(),
      duration_minutes: 240,
      motivation: 8,
      is_archived: false,
      created_at: now.toISOString(),
      updated_at: now.toISOString(),
      deleted_at: null,
    },
    {
      id: `guest_sample_2_${Date.now()}`,
      user_id: userId,
      sync_status: 'local_only',
      title: 'Review pull requests from team',
      description: null,
      status: 'todo',
      due: new Date(now.getTime() + 5 * day).toISOString(),
      duration_minutes: 90,
      motivation: 6,
      is_archived: false,
      created_at: now.toISOString(),
      updated_at: now.toISOString(),
      deleted_at: null,
    },
    {
      id: `guest_sample_3_${Date.now()}`,
      user_id: userId,
      sync_status: 'local_only',
      title: 'Update project documentation',
      description: 'Add API endpoints and migration guides',
      status: 'backlog',
      due: new Date(now.getTime() + 14 * day).toISOString(),
      duration_minutes: 180,
      motivation: 5,
      is_archived: false,
      created_at: now.toISOString(),
      updated_at: now.toISOString(),
      deleted_at: null,
    },
    {
      id: `guest_sample_4_${Date.now()}`,
      user_id: userId,
      sync_status: 'local_only',
      title: 'Deploy to production',
      description: 'After QA sign-off',
      status: 'done',
      due: new Date(now.getTime() - 1 * day).toISOString(),
      duration_minutes: 120,
      motivation: 9,
      is_archived: false,
      created_at: now.toISOString(),
      updated_at: now.toISOString(),
      deleted_at: null,
    },
    {
      id: `guest_sample_5_${Date.now()}`,
      user_id: userId,
      sync_status: 'local_only',
      title: 'High-priority bug fix',
      description: 'Fix authentication token expiration issue',
      status: 'inprogress',
      due: new Date(now.getTime() + 1 * day).toISOString(),
      duration_minutes: 150,
      motivation: 10,
      is_archived: false,
      created_at: now.toISOString(),
      updated_at: now.toISOString(),
      deleted_at: null,
    },
  ];

  return sampleItems;
}

let guestSeedInitialized = false;

// Auto-sync state shared across composable instances
let autoSyncTimer: ReturnType<typeof setInterval> | null = null;
let autoSyncInFlight = false;

// Reference count for concurrent loading operations
let loadingCount = 0;

// Session token for preventing race conditions on auth state changes
let currentLoadToken = 0;
let stopSyncStatusBinding: (() => void) | null = null;
type SyncStatus = 'pending' | 'success' | 'error' | undefined;
let boundSyncMapRef: Ref<Record<string, SyncStatus>> | null = null;
let previousSyncMap: Record<string, SyncStatus> = {};

/**
 * Composable for managing items/tasks
 * Handles fetching, creation, updates, and syncing with race condition protection
 */
export function useItems() {
  const auth = useAuth();

  // Watch for guest mode activation and seed sample items
  watch(
    () => [auth.isGuest.value, auth.userId.value],
    ([isGuest, userId]) => {
      if (isGuest && !guestSeedInitialized && userId) {
        items.value = generateGuestSeedItems(userId);
        guestSeedInitialized = true;
      } else if (!isGuest) {
        guestSeedInitialized = false;
      }
    },
    { immediate: true },
  );

  function createGuestItemId(): string {
    const randomPart = Math.random().toString(36).slice(2, 10);
    return `guest_item_${Date.now()}_${randomPart}`;
  }

  function createGuestNowIso(): string {
    return new Date().toISOString();
  }

  function reconcileSyncStatus(syncMap: Record<string, SyncStatus>): void {
    items.value.forEach((item) => {
      const status = syncMap[item.id];
      const previousStatus = previousSyncMap[item.id];
      if (status === 'success' && previousStatus === 'pending' && item.sync_status === 'modified') {
        item.sync_status = 'synced';
      } else if (status === 'pending' && item.sync_status !== 'local_only') {
        item.sync_status = 'modified';
      }
    });
    previousSyncMap = { ...syncMap };
  }

  function bindSyncStatusMap(syncMap: Ref<Record<string, SyncStatus>>): void {
    boundSyncMapRef = syncMap;
    previousSyncMap = {};
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
    if (auth.isGuest.value) {
      return items.value;
    }

    startLoading();
    error.value = null;
    try {
      const data = await itemRepository.getActiveItems();
      
      // Only update if this is the current session
      if (sessionToken === undefined || sessionToken === currentLoadToken) {
        items.value = data;
        if (boundSyncMapRef) {
          reconcileSyncStatus(boundSyncMapRef.value);
        }
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
    if (auth.isGuest.value) {
      return [];
    }

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
    if (auth.isGuest.value) {
      return [];
    }

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
    if (auth.isGuest.value) {
      return 0;
    }

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
    if (auth.isGuest.value) {
      return 0;
    }

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
    if (auth.isGuest.value) {
      return;
    }

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
    if (auth.isGuest.value) {
      const nowIso = createGuestNowIso();
      const id = createGuestItemId();
      const guestItem: Item = {
        id,
        user_id: auth.userId.value ?? 'guest_user',
        sync_status: 'local_only',
        title: payload.title,
        description: payload.description ?? null,
        status: 'todo',
        due: payload.due,
        duration_minutes: payload.durationMinutes ?? null,
        motivation: payload.motivation,
        is_archived: false,
        created_at: nowIso,
        updated_at: nowIso,
        deleted_at: null,
      };
      items.value = [guestItem, ...items.value];
      return id;
    }

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
    if (auth.isGuest.value) {
      const target = items.value.find((item) => item.id === payload.id);
      if (!target) {
        throw new Error('Item not found');
      }

      target.title = payload.title;
      target.description = payload.description ?? null;
      target.motivation = payload.motivation;
      target.due = payload.due;
      target.duration_minutes = payload.durationMinutes ?? null;
      target.sync_status = 'local_only';
      target.updated_at = createGuestNowIso();
      return;
    }

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
    if (auth.isGuest.value) {
      const item = items.value.find((entry) => entry.id === id);
      if (!item) {
        throw new Error('Item not found');
      }
      item.status = status;
      item.sync_status = 'local_only';
      item.updated_at = createGuestNowIso();
      return;
    }

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
    if (auth.isGuest.value) {
      items.value = items.value.filter((item) => item.id !== id);
      return;
    }

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
    if (auth.isGuest.value) {
      items.value = items.value.filter((item) => item.id !== id);
      return;
    }

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
