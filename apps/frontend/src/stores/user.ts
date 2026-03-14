import { ref, computed } from 'vue';
import { defineStore } from 'pinia';
import { authRepository } from '@/api/authRepository';

const USERNAME_STORAGE_KEY = 'taskgraph.username';

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

  function persistUsername(value: string | null): void {
    if (typeof localStorage === 'undefined') return;
    if (value) {
      localStorage.setItem(USERNAME_STORAGE_KEY, value);
    } else {
      localStorage.removeItem(USERNAME_STORAGE_KEY);
    }
  }

  function readStoredUsername(): string | null {
    if (typeof localStorage === 'undefined') return null;
    return normalizeUsername(localStorage.getItem(USERNAME_STORAGE_KEY));
  }

  // Actions
  async function initialize() {
    try {
      const storedUsername = readStoredUsername();

      // Try to auto-login using local_user table
      const localUser = await authRepository.autoLogin();
      
      if (localUser) {
        userId.value = localUser.id;
        username.value = normalizeUsername(localUser.username) ?? storedUsername;
        accessToken.value = localUser.access_token ?? null;
        if (!normalizeUsername(username.value)) {
          username.value = await resolveSessionUsername();
        }
        persistUsername(normalizeUsername(username.value));
        console.log('🔐 Auto-login successful');
      } else {
        // Fallback: Check active session
        const activeSession = await authRepository.getActiveSession();
        if (activeSession) {
          userId.value = activeSession.id;
          username.value = normalizeUsername(activeSession.username) ?? storedUsername;
          accessToken.value = activeSession.access_token ?? null;
          persistUsername(normalizeUsername(username.value));
        }
      }

      if (!normalizeUsername(username.value)) {
        username.value = storedUsername;
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
      username.value = normalizeUsername(response.username) ?? readStoredUsername();
      accessToken.value = response.access_token ?? null;
      if (!normalizeUsername(username.value)) {
        username.value = await resolveSessionUsername();
      }
      persistUsername(normalizeUsername(username.value));
      console.log('✅ Login successful');
    } catch (error) {
      console.error('Login failed:', error);
      throw error;
    }
  }

  async function logout() {
    try {
      await authRepository.logout();
      userId.value = null;
      username.value = null;
      accessToken.value = null;
      persistUsername(null);
    } catch (error) {
      console.error('Logout failed:', error);
      throw error;
    }
  }

  async function ensureUsername() {
    if (normalizeUsername(username.value)) {
      return username.value;
    }

    const storedUsername = readStoredUsername();
    if (storedUsername) {
      username.value = storedUsername;
      return storedUsername;
    }

    if (userId.value) {
      const sessionUsername = await resolveSessionUsername();
      if (sessionUsername) {
        username.value = sessionUsername;
        persistUsername(sessionUsername);
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
    login,
    logout,
  };
});
