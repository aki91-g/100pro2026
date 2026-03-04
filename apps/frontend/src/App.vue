<script setup lang="ts">
import { ref, onMounted } from "vue"; // Added onMounted here
import { invoke } from "@tauri-apps/api/core";

// 1. Import the new composable
import { useSyncStatus } from "@/composables/useSyncStatus";
const { syncMap, errorMap } = useSyncStatus();
import SyncButton from '@/components/SyncButton.vue';

// --- State Management ---
const items = ref<any[]>([]);
const greetMsg = ref("");
const backendMsg = ref("");
const isBackendLoading = ref(false);
const showDebugTools = ref(false);

// --- Lifecycle ---
onMounted(async () => {
  try {
    showDebugTools.value = await invoke("is_dev"); 
  } catch (e) {
    console.warn("Could not determine dev mode:", e);
    showDebugTools.value = false;
  }
});

// --- Desktop Bridge Logic (Rust + SQLite) ---
async function checkDatabase() {
  try {
    const data: any[] = await invoke("get_active_items");
    items.value = data; 
    greetMsg.value = `Connected! Showing ${data.length} tasks.`;
  } catch (e) {
    console.error("Fetch Error:", e);
    greetMsg.value = "Failed to load tasks.";
  }
}

async function seedDatabase() {
  try {
    await invoke("debug_seed_data");
    await checkDatabase();
    greetMsg.value = "Database seeded successfully!";
  } catch (e) {
    console.error("Rust Seed Error:", e);
    greetMsg.value = "Seed failed. Debug mode must be active.";
  }
}

async function resetDatabase() {
  if (!confirm("Are you sure? This will wipe all data!")) return;
  try {
    await invoke("debug_reset_db");
    items.value = []; // Clear local list immediately
    greetMsg.value = "Database wiped clean.";
  } catch (e) {
    console.error("Rust Reset Error:", e);
    greetMsg.value = "Failed to reset database.";
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
  <div class="container">
    <header>
      <h1>100pro2026 <span class="badge">Monorepo Active</span></h1>
      <SyncButton />
    </header>

    <main>
      <section class="card">
        <h2>1. Desktop Bridge (Rust + SQLite)</h2>
        <p class="description">Current connection to your local tasks.db.</p>
        <div class="input-group">
          <button @click="checkDatabase">Refresh List</button>
        </div>
        <div class="response-box" :class="{ hasValue: greetMsg }">
          {{ greetMsg || "Status unknown" }}
        </div>
      </section>

      <section class="card debug-section" v-if="showDebugTools">
        <h2 style="color: #d32f2f;">🛠 Debug Tools</h2>
        <p class="description">These tools are only visible in development builds.</p>
        <div class="input-group">
          <button @click="seedDatabase">Seed (Clean Slate)</button>
          <button @click="resetDatabase" class="btn-danger">Wipe Database</button>
        </div>
      </section>

      <section class="card" v-if="items.length > 0">
        <h2>📋 Current Tasks</h2>
        <div class="task-container">
          <div v-for="item in items" :key="item.id" class="task-row">
            <div class="sync-tag">
              <span v-if="syncMap[item.id] === 'pending'" class="dot pending">●</span>
              <span v-if="syncMap[item.id] === 'success'" class="dot success">Check</span>
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
.container { max-width: 600px; margin: 0 auto; padding: 2rem; font-family: 'Inter', sans-serif; }
header { text-align: center; margin-bottom: 2rem; }
.badge { font-size: 0.7rem; background: #42b883; color: white; padding: 4px 8px; border-radius: 12px; }

.card { background: #f8f9fa; border-radius: 12px; padding: 1.5rem; margin-bottom: 1.5rem; border: 1px solid #eee; }
.debug-section { border: 2px dashed #ffcdd2; background: #fff9f9; }

.input-group { display: flex; gap: 10px; margin-bottom: 1rem; }
button { background: #34495e; color: white; border: none; padding: 0.6rem 1.2rem; border-radius: 6px; cursor: pointer; }
button:hover { background: #41b883; }
.btn-danger { background: #e53935; }
.btn-danger:hover { background: #c62828; }

.response-box { min-height: 2.5rem; background: #fff; border: 1px dashed #ccc; border-radius: 6px; padding: 0.8rem; display: flex; align-items: center; justify-content: center; font-size: 0.9rem; color: #666; }
.response-box.hasValue { border-style: solid; border-color: #42b883; color: #2c3e50; background: #f0fff4; }

/* Task Styling */
.task-container { display: flex; flex-direction: column; gap: 8px; margin-top: 1rem; }
.task-row { display: flex; align-items: center; background: white; padding: 12px; border-radius: 8px; border: 1px solid #e0e0e0; }
.task-info { flex: 1; margin-left: 12px; text-align: left; }
.task-info p { margin: 2px 0 0 0; font-size: 0.8rem; color: #777; }
.status-pill { font-size: 0.65rem; font-weight: bold; padding: 3px 6px; border-radius: 4px; min-width: 70px; text-align: center; }

/* Status Colors */
.todo { background: #e3f2fd; color: #1976d2; }
.inprogress { background: #fff3e0; color: #f57c00; }
.done { background: #e8f5e9; color: #388e3c; }
.backlog { background: #f5f5f5; color: #616161; }
.motivation { color: #e53935; font-weight: bold; }

/* sync indicators */
.sync-tag { margin-right: 8px; font-size: 0.8rem; }
.dot.pending { color: #3498db; animation: blink 1s infinite; }
.dot.success { color: #42b883; font-weight: bold; }
.dot.error { color: #e53935; font-weight: bold; cursor: help; }
@keyframes blink {
  50% { opacity: 0; }
}

</style>

