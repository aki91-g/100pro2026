<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

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
let unlistenRemoteCatchup: UnlistenFn | null = null;

interface RemoteCatchupEvent {
  user_id: string;
  synced_count: number;
}

// Cancellation token for loadItems to prevent race conditions on logout
let currentLoadToken = 0;

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

  unlistenRemoteCatchup = await listen<RemoteCatchupEvent>("remote-catchup", (event) => {
    const { user_id, synced_count } = event.payload;
    if (userId.value && user_id === userId.value && isAuthenticated.value) {
      greetMsg.value = `Remote connected. Synced ${synced_count} local tasks to cloud.`;
    }
  });
});

onUnmounted(() => {
  if (unlistenRemoteCatchup) {
    unlistenRemoteCatchup();
    unlistenRemoteCatchup = null;
  }
});

// Watch for authentication changes
watch(isAuthenticated, async (authenticated) => {
  if (authenticated) {
    // Create a new token for this authentication session
    currentLoadToken++;
    const sessionToken = currentLoadToken;
    await loadItems(sessionToken);
  } else {
    // Invalidate any in-flight loadItems by incrementing token
    currentLoadToken++;
    items.value = [];
    greetMsg.value = "";
  }
}, { immediate: true });

// --- Desktop Bridge Logic (Rust + SQLite) ---
async function loadItems(sessionToken: number) {
  try {
    await fetchActiveItems();
    // Only commit results if this token is still current
    if (sessionToken === currentLoadToken) {
      greetMsg.value = `Connected! Showing ${items.value.length} tasks.`;
    }
  } catch (e) {
    // Only update error if token is still current
    if (sessionToken === currentLoadToken) {
      console.error("Fetch Error:", e);
      greetMsg.value = "Failed to load tasks.";
    }
  }
}

// Wrapper for manual refresh from button (no parameter needed)
async function handleRefreshItems() {
  await loadItems(currentLoadToken);
}

async function seedDatabase() {
  if (!isAuthenticated.value || !userId.value) {
    greetMsg.value = "Please login first to seed data.";
    return;
  }

  try {
    await seedDatabaseApi();
    const sessionToken = currentLoadToken;
    await loadItems(sessionToken);
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
    const message = String(e);
    if (message.includes("PostgreSQL is not connected")) {
      greetMsg.value = "Reset failed: remote database is offline. Please reconnect and try again.";
    } else {
      greetMsg.value = message || "Failed to reset database.";
    }
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
    const sessionToken = currentLoadToken;
    await loadItems(sessionToken);
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
          <button @click="handleRefreshItems">Refresh List</button>
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