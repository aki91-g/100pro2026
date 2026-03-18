import { storeToRefs } from 'pinia';
import { useUserStore } from '@/stores/user';

/**
 * Composable for authentication logic
 * Provides a clean API for auth operations
 */
export function useAuth() {
  const userStore = useUserStore();
  const refs = storeToRefs(userStore);

  function mapSignUpError(error: unknown): string {
    const message = String(error ?? '');

    if (message.includes('OFFLINE_REQUIRED_FOR_SIGNUP')) {
      return 'Creating an account requires an internet connection to sync your data.';
    }

    if (message.includes('SUPABASE_URL is not set') || message.includes('SUPABASE_ANON_KEY is not set')) {
      return 'Registration is temporarily unavailable. Please try again later.';
    }

    if (message.includes('Signup failed')) {
      const normalized = message.toLowerCase();
      if (normalized.includes('already') || normalized.includes('exists')) {
        return 'An account with this email already exists. Please log in.';
      }
      if (normalized.includes('password')) {
        return 'Your password does not meet requirements. Please use a stronger password.';
      }
      return 'Unable to create account. Please check your details and try again.';
    }

    if (message.includes('Invalid signup response')) {
      return 'Registration service returned an invalid response. Please try again.';
    }

    if (message.includes('Signup request failed')) {
      return 'Could not reach registration service. Please try again.';
    }

    return message;
  }

  async function signUp(email: string, password: string, username: string): Promise<void> {
    try {
      await userStore.signUp(email, password, username);
    } catch (error) {
      throw new Error(mapSignUpError(error));
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
