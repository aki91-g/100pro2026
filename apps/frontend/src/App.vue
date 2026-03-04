<script setup lang="ts">
import { ref, onMounted, watch } from "vue";

// Composables
import { useAuth } from "@/composables/useAuth";
import { useItems } from "@/composables/useItems";
import { useSyncStatus } from "@/composables/useSyncStatus";
import {
  fetchHonoHelloApi,
  isDevMode,
  migrateNullUserItemsApi,
  resetDatabaseApi,
  seedDatabaseApi,
} from "@/services/apiService";

// Components
import Login from "@/components/Login.vue";
import SyncButton from '@/components/SyncButton.vue';
import TaskList from "@/components/TaskList.vue";
import DebugTools from "@/components/DebugTools.vue";

// Auth
const { isAuthenticated, userId, logout, initialize } = useAuth();

// Items
const { items, fetchActiveItems, isSyncing } = useItems();

// Sync status
const { syncMap, errorMap } = useSyncStatus();

// Local state
const greetMsg = ref("");
const backendMsg = ref("");
const isBackendLoading = ref(false);
const showDebugTools = ref(false);

// --- Lifecycle ---
onMounted(async () => {
  // Initialize auth state
  await initialize();
  
  try {
    showDebugTools.value = await isDevMode();
  } catch (e) {
    console.warn("Could not determine dev mode:", e);
    showDebugTools.value = false;
  }
});

// Watch for authentication changes
watch(isAuthenticated, async (authenticated) => {
  if (authenticated) {
    await loadItems();
  } else {
    items.value = [];
    greetMsg.value = "";
  }
}, { immediate: true });

// --- Desktop Bridge Logic (Rust + SQLite) ---
async function loadItems() {
  try {
    await fetchActiveItems();
    greetMsg.value = `Connected! Showing ${items.value.length} tasks.`;
  } catch (e) {
    console.error("Fetch Error:", e);
    greetMsg.value = "Failed to load tasks.";
  }
}

async function seedDatabase() {
  if (!isAuthenticated.value || !userId.value) {
    greetMsg.value = "Please login first to seed data.";
    return;
  }

  try {
    await seedDatabaseApi();
    await loadItems();
    greetMsg.value = `Database seeded successfully for user '${userId.value}'!`;
  } catch (e) {
    console.error("Rust Seed Error:", e);
    greetMsg.value = String(e) || "Seed failed. Make sure database is empty first.";
  }
}

async function resetDatabase() {
  if (!isAuthenticated.value || !userId.value) {
    greetMsg.value = "Please login first to reset data.";
    return;
  }

  if (!confirm(`Are you sure? This will wipe all data for user '${userId.value}'!`)) return;
  try {
    await resetDatabaseApi();
    items.value = [];
    greetMsg.value = "Database wiped clean.";
  } catch (e) {
    console.error("Rust Reset Error:", e);
    greetMsg.value = String(e) || "Failed to reset database.";
  }
}

async function migrateNullUserItems() {
  if (!isAuthenticated.value || !userId.value) {
    greetMsg.value = "Please login first to migrate data.";
    return;
  }

  if (!confirm(`This will assign all items with NULL user_id to '${userId.value}'. Continue?`)) return;
  try {
    const count = await migrateNullUserItemsApi(true);
    greetMsg.value = `✓ Migrated ${count} items to your account.`;
    await loadItems();
  } catch (e) {
    console.error("Migration Error:", e);
    greetMsg.value = String(e) || "Migration failed.";
  }
}

async function handleLogout() {
  if (!confirm("Are you sure you want to logout?")) return;
  try {
    await logout();
    greetMsg.value = "";
  } catch (e) {
    console.error("Logout failed:", e);
  }
}

// --- Backend API Logic (Hono) ---
async function fetchFromHono() {
  isBackendLoading.value = true;
  try {
    const data = await fetchHonoHelloApi();
    backendMsg.value = `${data.message} (${new Date(data.timestamp).toLocaleTimeString()})`;
  } catch (e) {
    console.error("Hono Error:", e);
    backendMsg.value = "Hono connection failed. Check CORS or Server status.";
  } finally {
    isBackendLoading.value = false;
  }
}
</script>

<template>
  <!-- Show login screen if not authenticated -->
  <Login v-if="!isAuthenticated" />

  <!-- Main app content when authenticated -->
  <div v-else class="container">
    <header>
      <div class="header-content">
        <h1>100pro2026 <span class="badge">Monorepo Active</span></h1>
        <div class="user-info">
          <span class="user-id">👤 {{ userId }}</span>
          <button @click="handleLogout" class="logout-btn">Logout</button>
        </div>
      </div>
      <div class="sync-section">
        <SyncButton />
        <span v-if="isSyncing" class="syncing-indicator">🔄 Syncing...</span>
      </div>
    </header>

    <main>
      <section class="card">
        <h2>1. Desktop Bridge (Rust + SQLite)</h2>
        <p class="description">Current connection to your local tasks.db.</p>
        <div class="input-group">
          <button @click="loadItems">Refresh List</button>
        </div>
        <div class="response-box" :class="{ hasValue: greetMsg }">
          {{ greetMsg || "Status unknown" }}
        </div>
      </section>

      <DebugTools
        :visible="showDebugTools"
        :is-authenticated="isAuthenticated"
        :user-id="userId"
        @seed="seedDatabase"
        @reset="resetDatabase"
        @migrate="migrateNullUserItems"
      />

      <TaskList
        :items="items"
        :sync-map="syncMap"
        :error-map="errorMap"
      />

      <section class="card">
        <h2>2. Backend API (Hono)</h2>
        <p class="description">Communication with the shared Hono server.</p>
        <button @click="fetchFromHono" :disabled="isBackendLoading">
          {{ isBackendLoading ? "Connecting..." : "Ping Hono" }}
        </button>
        <div class="response-box" :class="{ hasValue: backendMsg }">
          {{ backendMsg || "Ready for request" }}
        </div>
      </section>
    </main>
  </div>
</template>

<style scoped>
.container { max-width: 700px; margin: 0 auto; padding: 2rem; font-family: 'Inter', sans-serif; }

header { margin-bottom: 2rem; }
.header-content { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; }
.sync-section { display: flex; justify-content: center; align-items: center; gap: 1rem; }

.user-info { display: flex; align-items: center; gap: 1rem; }
.user-id { font-size: 0.9rem; color: #666; font-weight: 500; }
.logout-btn { background: #e53935; padding: 0.4rem 0.8rem; font-size: 0.85rem; }
.logout-btn:hover { background: #c62828; }
.syncing-indicator { font-size: 0.85rem; color: #3498db; }

.badge { font-size: 0.7rem; background: #42b883; color: white; padding: 4px 8px; border-radius: 12px; }

.card { background: #f8f9fa; border-radius: 12px; padding: 1.5rem; margin-bottom: 1.5rem; border: 1px solid #eee; }
.description { color: #666; font-size: 0.9rem; margin-bottom: 1rem; }

.input-group { display: flex; gap: 10px; margin-bottom: 1rem; }
button { background: #34495e; color: white; border: none; padding: 0.6rem 1.2rem; border-radius: 6px; cursor: pointer; }
button:hover { background: #41b883; }
button:disabled { background: #999; cursor: not-allowed; }

.response-box { min-height: 2.5rem; background: #fff; border: 1px dashed #ccc; border-radius: 6px; padding: 0.8rem; display: flex; align-items: center; justify-content: center; font-size: 0.9rem; color: #666; }
.response-box.hasValue { border-style: solid; border-color: #42b883; color: #2c3e50; background: #f0fff4; }
</style>