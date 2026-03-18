import { storeToRefs } from 'pinia';
import { useUserStore } from '@/stores/user';

/**
 * Composable for authentication logic
 * Provides a clean API for auth operations
 */
export function useAuth() {
  const userStore = useUserStore();
  const refs = storeToRefs(userStore);

  async function signUp(email: string, password: string, username: string): Promise<void> {
    try {
      await userStore.signUp(email, password, username);
    } catch (error) {
      const message = String(error ?? '');
      if (message.includes('OFFLINE_REQUIRED_FOR_SIGNUP')) {
        throw new Error('Creating an account requires an internet connection to sync your data.');
      }
      throw error;
    }
  }

  return {
    // State
    userId: refs.userId,
    username: refs.username,
    displayUsername: refs.displayUsername,
    isAuthenticated: refs.isAuthenticated,
    isInitialized: refs.isInitialized,
    // Actions
    signUp,
    login: userStore.login,
    logout: userStore.logout,
    initialize: userStore.initialize,
    ensureUsername: userStore.ensureUsername,
  };
}
