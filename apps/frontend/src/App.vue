<script setup lang="ts">
import { onMounted, watch } from 'vue';

// Composables
import { useAuth } from '@/composables/useAuth';
import { useItems } from '@/composables/useItems';

// Components
import LoginView from '@/views/LoginView.vue';
import MainDashboard from '@/views/MainDashboard.vue';

// Auth
const { isAuthenticated, username, logout, initialize } = useAuth();

// Items (for session management)
const { startNewSession, invalidateSession } = useItems();

// --- Lifecycle ---
onMounted(async () => {
  // Initialize auth state (auto-login if session exists)
  await initialize();
});

// Watch for authentication changes
watch(
  isAuthenticated,
  async (authenticated) => {
    if (authenticated) {
      // Start a new session when user logs in
      startNewSession();
    } else {
      // Invalidate session and clear items when user logs out
      invalidateSession();
    }
  },
  { immediate: true }
);

// --- Logout Handler ---
async function handleLogout() {
  if (!confirm('Are you sure you want to logout?')) return;
  try {
    await logout();
  } catch (e) {
    console.error('Logout failed:', e);
  }
}
</script>

<template>
  <!-- Show login screen if not authenticated -->
  <LoginView v-if="!isAuthenticated" />

  <!-- Main app content when authenticated -->
  <div v-else class="app-wrapper">
    <header class="app-header">
      <div class="header-content">
        <h1>100pro2026 <span class="badge">Monorepo Active</span></h1>
        <div class="user-info">
          <span class="user-id">👤 {{ username || 'User' }}</span>
          <button @click="handleLogout" class="logout-btn">Logout</button>
        </div>
      </div>
    </header>

    <!-- Main Dashboard View -->
    <MainDashboard />
  </div>
</template>

<style scoped>
.app-wrapper {
  min-height: 100vh;
  font-family: 'Inter', sans-serif;
}

.app-header {
  max-width: 700px;
  margin: 0 auto;
  padding: 2rem 2rem 0;
}

.header-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}

.user-info {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.user-id {
  font-size: 0.9rem;
  color: #666;
  font-weight: 500;
}

.logout-btn {
  background: #e53935;
  color: white;
  border: none;
  padding: 0.4rem 0.8rem;
  font-size: 0.85rem;
  border-radius: 6px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.logout-btn:hover {
  background: #c62828;
}

.badge {
  font-size: 0.7rem;
  background: #42b883;
  color: white;
  padding: 4px 8px;
  border-radius: 12px;
}
</style>
