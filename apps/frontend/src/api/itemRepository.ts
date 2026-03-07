/**
 * Item Repository - Abstract API Layer
 * 
 * This repository provides a unified interface for item operations,
 * abstracting away whether we're using Tauri (local SQLite) or Hono (remote API).
 * 
 * The actual implementation is chosen at runtime based on the API mode.
 */

import { invoke } from '@tauri-apps/api/core';
import type { Item, RefreshResult } from '@/types/item';
import { getApiMode } from './config';
import { honoClient } from './honoClient';

export interface CreateItemPayload {
  title: string;
  motivation: number;
  due?: string | null;
  durationMinutes?: number | null;
}

export interface ItemRepository {
  getActiveItems(): Promise<Item[]>;
  getArchivedItems(): Promise<Item[]>;
  getDeletedItems(): Promise<Item[]>;
  createItem(payload: CreateItemPayload): Promise<string>;
  updateItemStatus(id: string, status: Item['status']): Promise<void>;
  archiveItem(id: string): Promise<void>;
  softDeleteItem(id: string): Promise<void>;
  syncItems(): Promise<number>;
  // Composite operations
  refreshItems(): Promise<RefreshResult>;
  syncAndRefresh(): Promise<{ count: number; data: RefreshResult }>;
}

/**
 * Tauri-based implementation (Local SQLite via Rust)
 */
class TauriItemRepository implements ItemRepository {
  async getActiveItems(): Promise<Item[]> {
    return invoke<Item[]>('get_active_items', {});
  }

  async getArchivedItems(): Promise<Item[]> {
    return invoke<Item[]>('get_archived_items', {});
  }

  async getDeletedItems(): Promise<Item[]> {
    return invoke<Item[]>('get_deleted_items', {});
  }

  async createItem(payload: CreateItemPayload): Promise<string> {
    return invoke<string>('create_item', { ...payload });
  }

  async updateItemStatus(id: string, status: Item['status']): Promise<void> {
    await invoke<void>('update_item_status', { id, status });
  }

  async archiveItem(id: string): Promise<void> {
    await invoke<void>('archive_item', { id });
  }

  async softDeleteItem(id: string): Promise<void> {
    await invoke<void>('soft_delete_item', { id });
  }

  async syncItems(): Promise<number> {
    return invoke<number>('sync_items', {});
  }

  /**
   * Fetches all items from the local SQLite database, categorized by state.
   */
  async refreshItems(): Promise<RefreshResult> {
    try {
      const [active, archived, deleted] = await Promise.all([
        this.getActiveItems(),
        this.getArchivedItems(),
        this.getDeletedItems(),
      ]);

      return { active, archived, deleted };
    } catch (err) {
      console.error('Failed to fetch items from local database:', err);
      throw err;
    }
  }

  /**
   * Triggers the Cloud -> Local sync engine in Rust.
   * 1. Pushes remote changes to local SQLite via UPSERT.
   * 2. Mirrors hard deletions from remote.
   * 3. Refreshes the local view.
   */
  async syncAndRefresh(): Promise<{ count: number; data: RefreshResult }> {
    try {
      // 1. Run the Rust sync logic
      const count = await this.syncItems();

      // 2. Immediately fetch the updated local state
      const data = await this.refreshItems();

      return { count, data };
    } catch (err) {
      console.error('Sync operation failed:', err);
      throw err;
    }
  }
}

/**
 * Hono-based implementation (Remote API)
 */
class HonoItemRepository implements ItemRepository {
  async getActiveItems(): Promise<Item[]> {
    return honoClient.getActiveItems();
  }

  async getArchivedItems(): Promise<Item[]> {
    return honoClient.getArchivedItems();
  }

  async getDeletedItems(): Promise<Item[]> {
    return honoClient.getDeletedItems();
  }

  async createItem(payload: CreateItemPayload): Promise<string> {
    return honoClient.createItem(payload);
  }

  async updateItemStatus(id: string, status: Item['status']): Promise<void> {
    await honoClient.updateItemStatus(id, status);
  }

  async archiveItem(id: string): Promise<void> {
    await honoClient.archiveItem(id);
  }

  async softDeleteItem(id: string): Promise<void> {
    await honoClient.softDeleteItem(id);
  }

  async syncItems(): Promise<number> {
    return honoClient.syncItems();
  }

  async refreshItems(): Promise<RefreshResult> {
    const [active, archived, deleted] = await Promise.all([
      this.getActiveItems(),
      this.getArchivedItems(),
      this.getDeletedItems(),
    ]);

    return { active, archived, deleted };
  }

  async syncAndRefresh(): Promise<{ count: number; data: RefreshResult }> {
    const count = await this.syncItems();
    const data = await this.refreshItems();
    return { count, data };
  }
}

/**
 * Factory function to get the appropriate repository implementation
 */
function createItemRepository(): ItemRepository {
  const mode = getApiMode();
  
  if (mode === 'hono') {
    console.log('🌐 Using Hono API client');
    return new HonoItemRepository();
  }
  
  console.log('🦀 Using Tauri local bridge');
  return new TauriItemRepository();
}

// Export singleton instance
export const itemRepository = createItemRepository();
