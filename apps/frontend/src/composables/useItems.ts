import { ref } from 'vue';
import type { Item } from '@/services/itemService';
import {
  archiveItemApi,
  createItemApi,
  fetchActiveItemsApi,
  fetchArchivedItemsApi,
  fetchDeletedItemsApi,
  softDeleteItemApi,
  syncItemsApi,
  updateItemStatusApi,
} from '@/services/apiService';

/**
 * Shared state for items across the application
 * Ensures all components see the same items/sync status
 */
const items = ref<Item[]>([]);
const isLoading = ref(false);
const error = ref<string | null>(null);
const isSyncing = ref(false);

// Reference count for concurrent loading operations
let loadingCount = 0;

/**
 * Composable for managing items/tasks
 * Handles fetching, creation, updates, and syncing
 */
export function useItems() {

  // Request counting for concurrency-safe isLoading
  function startLoading() {
    loadingCount++;
    isLoading.value = true;
  }

  function finishLoading() {
    loadingCount = Math.max(0, loadingCount - 1);
    isLoading.value = loadingCount > 0;
  }

  // Fetch active items
  async function fetchActiveItems() {
    startLoading();
    error.value = null;
    try {
      const data = await fetchActiveItemsApi();
      items.value = data;
      return data;
    } catch (err) {
      error.value = String(err);
      console.error('Failed to fetch active items:', err);
      throw err;
    } finally {
      finishLoading();
    }
  }

  // Fetch archived items
  async function fetchArchivedItems() {
    startLoading();
    error.value = null;
    try {
      const data = await fetchArchivedItemsApi();
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
  async function fetchDeletedItems() {
    startLoading();
    error.value = null;
    try {
      const data = await fetchDeletedItemsApi();
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
  async function syncItems() {
    isSyncing.value = true;
    error.value = null;
    try {
      const syncedCount = await syncItemsApi();
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

  // Create a new item
  async function createItem(
    title: string,
    motivation: number,
    due?: string | null,
    durationMinutes?: number | null
  ) {
    error.value = null;
    try {
      const id = await createItemApi({
        title,
        motivation,
        due,
        durationMinutes,
      });
      // Refresh items after creation
      await fetchActiveItems();
      return id;
    } catch (err) {
      error.value = String(err);
      console.error('Failed to create item:', err);
      throw err;
    }
  }

  // Update item status
  async function updateItemStatus(id: string, status: Item['status']) {
    error.value = null;
    try {
      await updateItemStatusApi(id, status);
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
  async function archiveItem(id: string) {
    error.value = null;
    try {
      await archiveItemApi(id);
      // Remove from local list
      items.value = items.value.filter((i) => i.id !== id);
    } catch (err) {
      error.value = String(err);
      console.error('Failed to archive item:', err);
      throw err;
    }
  }

  // Soft delete item
  async function softDeleteItem(id: string) {
    error.value = null;
    try {
      await softDeleteItemApi(id);
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
    // Actions
    fetchActiveItems,
    fetchArchivedItems,
    fetchDeletedItems,
    syncItems,
    createItem,
    updateItemStatus,
    archiveItem,
    softDeleteItem,
  };
}
