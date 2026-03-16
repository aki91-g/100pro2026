<script setup lang="ts">
import { onMounted, onUnmounted, watch } from 'vue';

// Composables
import { useAuth } from '@/composables/useAuth';
import { useItems } from '@/composables/useItems';

// Components
import LoginView from '@/views/LoginView.vue';
import MainDashboard from '@/views/MainDashboard.vue';

// Auth
const { isAuthenticated, initialize } = useAuth();

// Items (for session management and automated sync)
const { startNewSession, invalidateSession, startAutoSync, stopAutoSync } = useItems();

// --- Lifecycle ---
onMounted(async () => {
  // Initialize auth state (auto-login if session exists)
  await initialize();
});

onUnmounted(() => {
  stopAutoSync();
});

// Watch for authentication changes
watch(
  isAuthenticated,
  async (authenticated) => {
    if (authenticated) {
      // Start a new session and background sync loop when user logs in
      const sessionToken = startNewSession();
      startAutoSync(sessionToken);
    } else {
      // Stop sync loop, invalidate session, and clear items on logout
      stopAutoSync();
      invalidateSession();
    }
  },
  { immediate: true }
);

</script>

<template>
  <!-- Show login screen if not authenticated -->
  <LoginView v-if="!isAuthenticated" />

  <!-- Main app content when authenticated -->
  <MainDashboard v-else />
</template>
