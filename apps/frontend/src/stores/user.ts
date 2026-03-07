import { ref, computed } from 'vue';
import { defineStore } from 'pinia';
import { authRepository } from '@/api/authRepository';

export const useUserStore = defineStore('user', () => {
  // State
  const userId = ref<string | null>(null);
  const username = ref<string | null>(null);
  const isInitialized = ref(false);

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
        console.log(`🔐 Auto-login successful: ${username.value}`);
      } else {
        // Fallback: Check active session
        const activeSession = await authRepository.getActiveSession();
        if (activeSession) {
          userId.value = activeSession.id;
          username.value = activeSession.username;
        }
      }
      
      isInitialized.value = true;
    } catch (error) {
      console.error('Failed to initialize user:', error);
      userId.value = null;
      username.value = null;
      isInitialized.value = true;
    }
  }

  async function login(email: string, password: string) {
    try {
      // Auth repository handles authentication
      const response = await authRepository.login(email, password);
      
      userId.value = response.id;
      username.value = response.username;
      
      console.log(`✅ Login successful: ${response.username}`);
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
    // Getters
    isAuthenticated,
    // Actions
    initialize,
    login,
    logout,
  };
});
