import { storeToRefs } from 'pinia';
import { useUserStore } from '@/stores/user';

/**
 * Composable for authentication logic
 * Provides a clean API for auth operations
 */
export function useAuth() {
  const userStore = useUserStore();
  const { userId, isAuthenticated, isInitialized } = storeToRefs(userStore);

  return {
    // State
    userId,
    isAuthenticated,
    isInitialized,
    // Actions
    login: userStore.login,
    logout: userStore.logout,
    initialize: userStore.initialize,
  };
}
