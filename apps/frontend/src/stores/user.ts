import { ref, computed } from 'vue';
import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';

export const useUserStore = defineStore('user', () => {
  // State
  const userId = ref<string | null>(null);
  const isInitialized = ref(false);

  // Getters
  const isAuthenticated = computed(() => userId.value !== null);

  // Actions
  async function initialize() {
    try {
      const currentUser = await invoke<string | null>('get_current_user');
      userId.value = currentUser;
      isInitialized.value = true;
    } catch (error) {
      console.error('Failed to get current user:', error);
      userId.value = null;
      isInitialized.value = true;
    }
  }

  async function login(newUserId: string) {
    try {
      await invoke('set_user', { userId: newUserId });
      userId.value = newUserId;
      
      // Claim any offline items
      try {
        const claimedCount = await invoke<number>('claim_offline_items');
        console.log(`Claimed ${claimedCount} offline items for user ${newUserId}`);
      } catch (claimError) {
        console.error('Failed to claim offline items:', claimError);
      }
    } catch (error) {
      console.error('Login failed:', error);
      throw error;
    }
  }

  async function logout() {
    try {
      await invoke('clear_user');
      userId.value = null;
    } catch (error) {
      console.error('Logout failed:', error);
      throw error;
    }
  }

  return {
    // State
    userId,
    isInitialized,
    // Getters
    isAuthenticated,
    // Actions
    initialize,
    login,
    logout,
  };
});
