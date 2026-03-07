import { ref, computed } from 'vue';
import { defineStore } from 'pinia';
import { authRepository } from '@/api/authRepository';

export const useUserStore = defineStore('user', () => {
  // State
  const userId = ref<string | null>(null);
  const username = ref<string | null>(null);
  const isInitialized = ref(false);
  const accessToken = ref<string | null>(null);

  // Getters
  const isAuthenticated = computed(() => userId.value !== null);

  // Actions
  async function initialize() {
    try {
      // Try to auto-login using local_user table
      const localUser = await authRepository.autoLogin();
      
      if (localUser) {
        userId.value = localUser.id;
        username.value = localUser.username;
        accessToken.value = localUser.access_token ?? null;
        console.log('🔐 Auto-login successful');
      } else {
        // Fallback: Check active session
        const activeSession = await authRepository.getActiveSession();
        if (activeSession) {
          userId.value = activeSession.id;
          username.value = activeSession.username;
          accessToken.value = activeSession.access_token ?? null;
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
      username.value = response.username;
      accessToken.value = response.access_token ?? null;
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
    } catch (error) {
      console.error('Logout failed:', error);
      throw error;
    }
  }

  return {
    // State
    userId,
    username,
    isInitialized,
    accessToken,
    // Getters
    isAuthenticated,
    // Actions
    initialize,
    login,
    logout,
  };
});
