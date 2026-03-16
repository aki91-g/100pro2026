import { storeToRefs } from 'pinia';
import { useUserStore } from '@/stores/user';

/**
 * Composable for authentication logic
 * Provides a clean API for auth operations
 */
export function useAuth() {
  const userStore = useUserStore();
  const refs = storeToRefs(userStore);

  return {
    // State
    userId: refs.userId,
    username: refs.username,
    displayUsername: refs.displayUsername,
    isAuthenticated: refs.isAuthenticated,
    isInitialized: refs.isInitialized,
    // Actions
    login: userStore.login,
    logout: userStore.logout,
    initialize: userStore.initialize,
    ensureUsername: userStore.ensureUsername,
  };
}
