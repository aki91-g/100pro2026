import { invoke } from '@tauri-apps/api/core';
import { getApiMode } from './config';
import { honoClient } from './honoClient';

/**
 * Abstract debug repository interface
 */
export interface DebugRepository {
  isDevMode(): Promise<boolean>;
  seedDatabase(): Promise<void>;
  resetDatabase(): Promise<void>;
  migrateNullUserItems(assignToCurrentUser: boolean): Promise<number>;
}

/**
 * Tauri implementation using Rust backend commands
 */
export class TauriDebugRepository implements DebugRepository {
  async isDevMode(): Promise<boolean> {
    return invoke<boolean>('is_dev', {});
  }

  async seedDatabase(): Promise<void> {
    await invoke<void>('debug_seed_data', {});
  }

  async resetDatabase(): Promise<void> {
    await invoke<void>('debug_reset_db', {});
  }

  async migrateNullUserItems(assignToCurrentUser: boolean): Promise<number> {
    return invoke<number>('debug_migrate_null_users', { assignToCurrentUser });
  }
}

/**
 * Hono implementation using REST API
 */
export class HonoDebugRepository implements DebugRepository {
  async isDevMode(): Promise<boolean> {
    const response = await honoClient.get('/api/debug/dev-mode');
    const data = await response.json();
    return data.isDevMode;
  }

  async seedDatabase(): Promise<void> {
    await honoClient.post('/api/debug/seed', {});
  }

  async resetDatabase(): Promise<void> {
    await honoClient.post('/api/debug/reset', {});
  }

  async migrateNullUserItems(assignToCurrentUser: boolean): Promise<number> {
    const response = await honoClient.post('/api/debug/migrate', {
      assignToCurrentUser,
    });
    const data = await response.json();
    return data.count;
  }
}

/**
 * Factory function to create the appropriate repository based on the API mode
 */
export function createDebugRepository(): DebugRepository {
  const mode = getApiMode();

  if (mode === 'tauri') {
    return new TauriDebugRepository();
  }

  return new HonoDebugRepository();
}

// Export a singleton instance
export const debugRepository = createDebugRepository();
