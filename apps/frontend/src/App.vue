<script setup lang="ts">
import { onMounted, onUnmounted, watch } from 'vue';

// Composables
import { useAuth } from '@/composables/useAuth';
import { useItems } from '@/composables/useItems';

// Components
import LoginView from '@/views/LoginView.vue';
import MainDashboard from '@/views/MainView.vue';

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

watch(
  isAuthenticated,
  async (authenticated) => {
    if (authenticated) {
      const sessionToken = startNewSession();
      startAutoSync(sessionToken);
    } else {
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
