import { ref, computed } from 'vue';
import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';

interface LocalUser {
  id: string;
  username: string;
  last_login: string | null;
  is_active: number;
}

interface LoginResponse {
  id: string;
  username: string;
}

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
      const localUser = await invoke<LocalUser | null>('auto_login');
      
      if (localUser) {
        userId.value = localUser.id;
        username.value = localUser.username;
        console.log(`🔐 Auto-login successful: ${username.value}`);
      } else {
        // Fallback: Check current user from AppState
        const currentUser = await invoke<string | null>('get_current_user');
        userId.value = currentUser;
        
        // Try to get username from active local user
        if (currentUser) {
          try {
            const activeUser = await invoke<LocalUser | null>('get_active_local_user');
            if (activeUser) {
              username.value = activeUser.username;
            }
          } catch (err) {
            console.warn('Could not fetch active local user:', err);
          }
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
      // Rust handles Supabase auth and resolves identity
      const response = await invoke<LoginResponse>('login', { 
        email,
        password,
      });
      
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
      await invoke('logout');
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
