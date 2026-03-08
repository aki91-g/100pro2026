import { ref } from 'vue';
import { debugRepository, type HonoHelloResponse } from '@/api/debugRepository';

/**
 * Composable for debug operations
 * Provides a clean API for development/testing operations
 */
export function useDebug() {
  const isDevMode = ref(false);
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  /**
   * Check if the app is running in development mode
   */
  async function checkDevMode(): Promise<boolean> {
    try {
      isDevMode.value = await debugRepository.isDevMode();
      return isDevMode.value;
    } catch (err) {
      console.error('Failed to check dev mode:', err);
      isDevMode.value = false;
      return false;
    }
  }

  /**
   * Seed the database with sample data
   */
  async function seedDatabase(): Promise<void> {
    isLoading.value = true;
    error.value = null;

    try {
      await debugRepository.seedDatabase();
    } catch (err) {
      error.value = String(err);
      throw err;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Reset/wipe the database
   */
  async function resetDatabase(): Promise<void> {
    isLoading.value = true;
    error.value = null;

    try {
      await debugRepository.resetDatabase();
    } catch (err) {
      error.value = String(err);
      throw err;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Migrate items with NULL user_id to the current user
   */
  async function migrateNullUserItems(assignToCurrentUser: boolean): Promise<number> {
    isLoading.value = true;
    error.value = null;

    try {
      return await debugRepository.migrateNullUserItems(assignToCurrentUser);
    } catch (err) {
      error.value = String(err);
      throw err;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Fetch hello message from Hono backend for connectivity testing
   */
  async function fetchHonoHello(): Promise<HonoHelloResponse> {
    error.value = null;

    try {
      return await debugRepository.fetchHonoHello();
    } catch (err) {
      error.value = String(err);
      throw err;
    }
  }

  return {
    // State
    isDevMode,
    isLoading,
    error,
    // Actions
    checkDevMode,
    seedDatabase,
    resetDatabase,
    migrateNullUserItems,
    fetchHonoHello,
  };
}
