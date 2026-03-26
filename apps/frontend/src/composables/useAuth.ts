import { storeToRefs } from 'pinia';
import { computed, ref } from 'vue';
import { useUserStore } from '@/stores/user';

const GUEST_SESSION_TOKEN = 'guest-session-token';

const isGuest = ref(false);
const guestUserId = ref<string | null>(null);
const guestUsername = ref<string | null>(null);
const guestAccessToken = ref<string | null>(null);

function createGuestUsername(): string {
  const suffix = Math.floor(1000 + Math.random() * 9000);
  return `Guest_${suffix}`;
}

function createGuestUserId(): string {
  const randomPart = Math.random().toString(36).slice(2, 10);
  return `guest_${Date.now()}_${randomPart}`;
}

function clearGuestState(): void {
  isGuest.value = false;
  guestUserId.value = null;
  guestUsername.value = null;
  guestAccessToken.value = null;
}

/**
 * Composable for authentication logic
 * Provides a clean API for auth operations
 */
export function useAuth() {
  const userStore = useUserStore();
  const refs = storeToRefs(userStore);

  const userId = computed(() => (isGuest.value ? guestUserId.value : refs.userId.value));
  const username = computed(() => (isGuest.value ? guestUsername.value : refs.username.value));
  const displayUsername = computed(() => {
    if (isGuest.value) {
      return guestUsername.value ?? 'Guest';
    }
    return refs.displayUsername.value;
  });
  const isAuthenticated = computed(() => isGuest.value || refs.isAuthenticated.value);

  function mapSignUpError(error: unknown): string {
    const rawMessage = String(error ?? '');
    const message = rawMessage
      .replace(/^post\s*\/api\/auth\/signup\s*failed\s*:\s*/i, '')
      .trim();

    if (message.includes('OFFLINE_REQUIRED_FOR_SIGNUP')) {
      return 'Creating an account requires an internet connection to sync your data.';
    }

    if (message.includes('SUPABASE_URL is not set') || message.includes('SUPABASE_ANON_KEY is not set')) {
      return 'Registration is temporarily unavailable. Please try again later.';
    }

    if (message.includes('Registration succeeded but no active session was returned')) {
      return 'Please confirm your email to log in.';
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
      clearGuestState();
      await userStore.signUp(email, password, username);
    } catch (error) {
      throw new Error(mapSignUpError(error));
    }
  }

  async function login(email: string, password: string): Promise<void> {
    clearGuestState();
    await userStore.login(email, password);
  }

  async function logout(): Promise<void> {
    if (isGuest.value) {
      clearGuestState();
      return;
    }

    await userStore.logout();
  }

  async function initialize(): Promise<void> {
    if (isGuest.value) {
      return;
    }
    await userStore.initialize();
  }

  async function ensureUsername(): Promise<string> {
    if (isGuest.value) {
      return guestUsername.value ?? 'Guest';
    }
    const resolved = await userStore.ensureUsername();
    return resolved ?? 'User';
  }

  function continueAsGuest(): void {
    isGuest.value = true;
    guestUserId.value = createGuestUserId();
    guestUsername.value = createGuestUsername();
    guestAccessToken.value = GUEST_SESSION_TOKEN;
  }

  return {
    // State
    userId,
    username,
    displayUsername,
    isAuthenticated,
    isGuest,
    accessToken: computed(() => (isGuest.value ? guestAccessToken.value : refs.accessToken.value)),
    isInitialized: refs.isInitialized,
    // Actions
    signUp,
    login,
    logout,
    initialize,
    ensureUsername,
    continueAsGuest,
  };
}
