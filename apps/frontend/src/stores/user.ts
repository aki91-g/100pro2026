import { ref, computed } from 'vue';
import { defineStore } from 'pinia';
import { authRepository } from '@/api/authRepository';

const USERNAME_STORAGE_PREFIX = 'taskgraph.username';

function normalizeUsername(value: string | null | undefined): string | null {
  if (!value) return null;
  const trimmed = value.trim();
  if (!trimmed) return null;
  if (trimmed.toLowerCase() === 'unknown') return null;
  return trimmed;
}

async function resolveSessionUsername(): Promise<string | null> {
  try {
    const activeSession = await authRepository.getActiveSession();
    return normalizeUsername(activeSession?.username);
  } catch {
    return null;
  }
}

export const useUserStore = defineStore('user', () => {
  // State
  const userId = ref<string | null>(null);
  const username = ref<string | null>(null);
  const isInitialized = ref(false);
  const accessToken = ref<string | null>(null);

  // Getters
  const isAuthenticated = computed(() => userId.value !== null);
  const displayUsername = computed(() => normalizeUsername(username.value) ?? 'User');

  function getUsernameStorageKey(userId: string): string {
    return `${USERNAME_STORAGE_PREFIX}:${userId}`;
  }

  function persistUsername(userId: string | null, value: string | null): void {
    if (typeof localStorage === 'undefined') return;
    if (!userId) return;

    const key = getUsernameStorageKey(userId);
    if (value && value.length > 0) {
      localStorage.setItem(key, value);
    } else {
      localStorage.removeItem(key);
    }
  }

  function readStoredUsername(userId: string | null): string | null {
    if (typeof localStorage === 'undefined') return null;
    if (!userId) return null;
    return normalizeUsername(localStorage.getItem(getUsernameStorageKey(userId)));
  }

  // Actions
  async function initialize() {
    try {
      // Try to auto-login using local_user table
      const localUser = await authRepository.autoLogin();
      
      if (localUser) {
        userId.value = localUser.id;
        const storedUsername = readStoredUsername(localUser.id);
        username.value = normalizeUsername(localUser.username);
        accessToken.value = localUser.access_token ?? null;
        if (!normalizeUsername(username.value)) {
          username.value = (await resolveSessionUsername()) ?? storedUsername;
        }
        persistUsername(userId.value, normalizeUsername(username.value));
        console.log('🔐 Auto-login successful');
      } else {
        // Fallback: Check active session
        const activeSession = await authRepository.getActiveSession();
        if (activeSession) {
          userId.value = activeSession.id;
          const storedUsername = readStoredUsername(activeSession.id);
          username.value = normalizeUsername(activeSession.username) ?? storedUsername;
          accessToken.value = activeSession.access_token ?? null;
          persistUsername(userId.value, normalizeUsername(username.value));
        }
      }
      
      isInitialized.value = true;
    } catch (error) {
      console.error('Failed to initialize user:', error);
      userId.value = null;
      username.value = null;
      accessToken.value = null;
      isInitialized.value = true;
      
    }
  }

  async function login(email: string, password: string) {
    try {
      // Auth repository handles authentication
      const response = await authRepository.login(email, password);
      
      userId.value = response.id;
      const storedUsername = readStoredUsername(response.id);
      username.value = normalizeUsername(response.username) ?? storedUsername;
      accessToken.value = response.access_token ?? null;
      if (!normalizeUsername(username.value)) {
        username.value = await resolveSessionUsername();
      }
      persistUsername(userId.value, normalizeUsername(username.value));
      console.log('✅ Login successful');
    } catch (error) {
      console.error('Login failed:', error);
      throw error;
    }
  }

  async function signUp(email: string, password: string, preferredUsername: string) {
    try {
      const response = await authRepository.signUp(email, password, preferredUsername);

      userId.value = response.id;
      const storedUsername = readStoredUsername(response.id);
      const normalizedResponseUsername = normalizeUsername(response.username);
      const normalizedPreferredUsername = normalizeUsername(preferredUsername);
      username.value = normalizedResponseUsername ?? normalizedPreferredUsername ?? storedUsername;
      accessToken.value = response.access_token ?? null;

      if (!normalizeUsername(username.value)) {
        username.value = await resolveSessionUsername();
      }

      persistUsername(userId.value, normalizeUsername(username.value));
      console.log('Registration successful');
    } catch (error) {
      console.error('Registration failed:', error);
      throw error;
    }
  }

  async function logout() {
    try {
      await authRepository.logout();
      const currentUserId = userId.value;
      userId.value = null;
      username.value = null;
      accessToken.value = null;
      persistUsername(currentUserId, null);
    } catch (error) {
      console.error('Logout failed:', error);
      throw error;
    }
  }

  async function ensureUsername() {
    if (normalizeUsername(username.value)) {
      return username.value;
    }

    const storedUsername = readStoredUsername(userId.value);
    if (storedUsername) {
      username.value = storedUsername;
      return storedUsername;
    }

    if (userId.value) {
      const sessionUsername = await resolveSessionUsername();
      if (sessionUsername) {
        username.value = sessionUsername;
        persistUsername(userId.value, sessionUsername);
        return sessionUsername;
      }
    }

    await initialize();
    return normalizeUsername(username.value) ?? 'User';
  }

  return {
    // State
    userId,
    username,
    isInitialized,
    accessToken,
    // Getters
    isAuthenticated,
    displayUsername,
    // Actions
    initialize,
    ensureUsername,
    signUp,
    login,
    logout,
  };
});
