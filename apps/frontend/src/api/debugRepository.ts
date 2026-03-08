import { invoke } from '@tauri-apps/api/core';
import { getApiMode } from './config';
import { honoClient } from './honoClient';

export interface HonoHelloResponse {
  message: string;
  timestamp: string;
}

/**
 * Abstract debug repository interface
 */
export interface DebugRepository {
  isDevMode(): Promise<boolean>;
  seedDatabase(): Promise<void>;
  resetDatabase(): Promise<void>;
  migrateNullUserItems(assignToCurrentUser: boolean): Promise<number>;
  fetchHonoHello(): Promise<HonoHelloResponse>;
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

  async fetchHonoHello(): Promise<HonoHelloResponse> {
    // Tauri can still call the Hono API for testing purposes
    const honoBaseUrl = import.meta.env.VITE_HONO_BASE_URL || 'http://localhost:3000';
    const url = `${honoBaseUrl}/api/hello`;
    
    // Create abort controller with 5 second timeout
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), 5000);

    try {
      const response = await fetch(url, {
        signal: controller.signal,
      });

      if (!response.ok) {
        throw new Error('Network response was not ok');
      }

      return response.json() as Promise<HonoHelloResponse>;
    } finally {
      clearTimeout(timeout);
    }
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

  async fetchHonoHello(): Promise<HonoHelloResponse> {
    const response = await honoClient.get('/api/hello');
    return response.json();
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
