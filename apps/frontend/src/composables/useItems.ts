import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Item } from '@/services/itemService';

/**
 * Shared state for items across the application
 * Ensures all components see the same items/sync status
 */
const items = ref<Item[]>([]);
const isLoading = ref(false);
const error = ref<string | null>(null);
const isSyncing = ref(false);

/**
 * Composable for managing items/tasks
 * Handles fetching, creation, updates, and syncing
 */
export function useItems() {

  // Fetch active items
  async function fetchActiveItems() {
    isLoading.value = true;
    error.value = null;
    try {
      const data = await invoke<Item[]>('get_active_items');
      items.value = data;
      return data;
    } catch (err) {
      error.value = String(err);
      console.error('Failed to fetch active items:', err);
      throw err;
    } finally {
      isLoading.value = false;
    }
  }

  // Fetch archived items
  async function fetchArchivedItems() {
    isLoading.value = true;
    error.value = null;
    try {
      const data = await invoke<Item[]>('get_archived_items');
      return data;
    } catch (err) {
      error.value = String(err);
      console.error('Failed to fetch archived items:', err);
      throw err;
    } finally {
      isLoading.value = false;
    }
  }

  // Fetch deleted items
  async function fetchDeletedItems() {
    isLoading.value = true;
    error.value = null;
    try {
      const data = await invoke<Item[]>('get_deleted_items');
      return data;
    } catch (err) {
      error.value = String(err);
      console.error('Failed to fetch deleted items:', err);
      throw err;
    } finally {
      isLoading.value = false;
    }
  }

  // Sync with remote
  async function syncItems() {
    isSyncing.value = true;
    error.value = null;
    try {
      const syncedCount = await invoke<number>('sync_items');
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
    try {
      const id = await invoke<string>('create_item', {
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
    try {
      await invoke('update_item_status', { id, status });
      // Update local state
      const item = items.value.find((i) => i.id === id);
      if (item) {
        item.status = status;
      }
    } catch (err) {
      error.value = String(err);
      console.error('Failed to update item status:', err);
      throw err;
    }
  }

  // Archive item
  async function archiveItem(id: string) {
    try {
      await invoke('archive_item', { id });
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
    try {
      await invoke('soft_delete_item', { id });
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
