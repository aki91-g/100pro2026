<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";

// Composables
import { useAuth } from "@/composables/useAuth";
import { useItems } from "@/composables/useItems";
import { useSyncStatus } from "@/composables/useSyncStatus";

// Components
import Login from "@/components/Login.vue";
import SyncButton from '@/components/SyncButton.vue';

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
    showDebugTools.value = await invoke("is_dev"); 
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
    await invoke("debug_seed_data");
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
    await invoke("debug_reset_db");
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
    const count = await invoke<number>("debug_migrate_null_users", { assignToCurrentUser: true });
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
    const res = await fetch("http://localhost:3000/api/hello");
    if (!res.ok) throw new Error("Network response was not ok");
    const data = await res.json();
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

      <section class="card debug-section" v-if="showDebugTools">
        <h2 style="color: #d32f2f;">🛠 Debug Tools</h2>
        <p class="description">These tools are only visible in development builds.</p>
        <p class="description" v-if="isAuthenticated" style="color: #42b883;">
          ✓ Logged in as: <strong>{{ userId }}</strong>
        </p>
        <p class="description" v-else style="color: #f57c00;">
          ⚠ Login required to use debug tools
        </p>
        <div class="input-group">
          <button @click="seedDatabase" :disabled="!isAuthenticated">Seed Demo Data</button>
          <button @click="resetDatabase" :disabled="!isAuthenticated" class="btn-danger">Wipe My Data</button>
        </div>
        <div class="input-group" style="margin-top: 0.5rem;">
          <button @click="migrateNullUserItems" :disabled="!isAuthenticated" class="btn-migration">
            🔄 Claim Orphaned Items
          </button>
        </div>
        <p class="description" style="font-size: 0.75rem; color: #666; margin-top: 0.5rem;">
          💡 Tip: "Claim Orphaned Items" assigns items with NULL user_id to your account.
        </p>
      </section>

      <section class="card" v-if="items.length > 0">
        <h2>📋 Current Tasks</h2>
        <div class="task-container">
          <div v-for="item in items" :key="item.id" class="task-row">
            <div class="sync-tag">
              <span v-if="syncMap[item.id] === 'pending'" class="dot pending">●</span>
              <span v-if="syncMap[item.id] === 'success'" class="dot success">✓</span>
              <span v-if="syncMap[item.id] === 'error'" class="dot error" :title="errorMap[item.id]">!</span>
            </div>

            <span :class="['status-pill', item.status.toLowerCase()]">
              {{ item.status }}
            </span>
            
            <div class="task-info">
              <strong>{{ item.title }}</strong>
              <p v-if="item.description">{{ item.description }}</p>
            </div>

            <div class="task-meta">
              <span class="motivation">🔥 {{ item.motivation }}</span>
            </div>
          </div>
        </div>
      </section>

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
.debug-section { border: 2px dashed #ffcdd2; background: #fff9f9; }
.description { color: #666; font-size: 0.9rem; margin-bottom: 1rem; }

.input-group { display: flex; gap: 10px; margin-bottom: 1rem; }
button { background: #34495e; color: white; border: none; padding: 0.6rem 1.2rem; border-radius: 6px; cursor: pointer; }
button:hover { background: #41b883; }
button:disabled { background: #999; cursor: not-allowed; }
.btn-danger { background: #e53935; }
.btn-danger:hover { background: #c62828; }
.btn-migration { background: #3498db; }
.btn-migration:hover { background: #2980b9; }
.btn-migration:disabled { background: #999; }

.response-box { min-height: 2.5rem; background: #fff; border: 1px dashed #ccc; border-radius: 6px; padding: 0.8rem; display: flex; align-items: center; justify-content: center; font-size: 0.9rem; color: #666; }
.response-box.hasValue { border-style: solid; border-color: #42b883; color: #2c3e50; background: #f0fff4; }

/* Task Styling */
.task-container { display: flex; flex-direction: column; gap: 8px; margin-top: 1rem; }
.task-row { display: flex; align-items: center; background: white; padding: 12px; border-radius: 8px; border: 1px solid #e0e0e0; }
.task-info { flex: 1; margin-left: 12px; text-align: left; }
.task-info p { margin: 2px 0 0 0; font-size: 0.8rem; color: #777; }
.task-meta { display: flex; align-items: center; gap: 8px; }
.status-pill { font-size: 0.65rem; font-weight: bold; padding: 3px 6px; border-radius: 4px; min-width: 70px; text-align: center; }

/* Status Colors */
.todo { background: #e3f2fd; color: #1976d2; }
.inprogress { background: #fff3e0; color: #f57c00; }
.done { background: #e8f5e9; color: #388e3c; }
.backlog { background: #f5f5f5; color: #616161; }
.motivation { color: #e53935; font-weight: bold; font-size: 0.85rem; }

/* sync indicators */
.sync-tag { margin-right: 8px; font-size: 0.8rem; }
.dot.pending { color: #3498db; animation: blink 1s infinite; }
.dot.success { color: #42b883; font-weight: bold; }
.dot.error { color: #e53935; font-weight: bold; cursor: help; }
@keyframes blink {
  50% { opacity: 0; }
}
</style>