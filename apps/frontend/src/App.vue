<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

// --- 状態管理 ---
const items = ref<any[]>([]);
const greetMsg = ref("");
const backendMsg = ref("");
const isBackendLoading = ref(false);

// --- ロジック: Rust (Desktop) との通信 ---
async function checkDatabase() {
  try {
    const data: any[] = await invoke("get_active_items");
    items.value = data; // Store the array
    greetMsg.value = `Connected! Showing ${data.length} tasks.`;
  } catch (e) {
    console.error("Fetch Error:", e);
    greetMsg.value = "Failed to load tasks.";
  }
}

async function seedDatabase() {
  try {
    // Rustの 'debug_seed_data' コマンドを呼び出し
    await invoke("debug_seed_data");
    
    // シード後に件数を再取得して表示を更新
    await checkDatabase();
    greetMsg.value = "Database seeded successfully!";
  } catch (e) {
    console.error("Rust Seed Error:", e);
    greetMsg.value = "Failed to seed database. (Check if debug mode is active)";
  }
}

async function resetDatabase() {
  if (!confirm("Are you sure? This will wipe all data!")) return;
  
  try {
    // Rustの 'debug_reset_db' コマンドを呼び出し
    await invoke("debug_reset_db");
    
    // リセット後に件数を再取得
    await checkDatabase();
    greetMsg.value = "Database has been reset to factory settings.";
  } catch (e) {
    console.error("Rust Reset Error:", e);
    greetMsg.value = "Failed to reset database.";
  }
}

// --- ロジック: Hono (Backend) との通信 ---
async function fetchFromHono() {
  isBackendLoading.value = true;
  try {
    // Hono サーバー (localhost:3000) へリクエスト
    const res = await fetch("http://localhost:3000/api/hello");
    if (!res.ok) throw new Error("Network response was not ok");
    
    const data = await res.json();
    backendMsg.value = `${data.message} (取得時刻: ${new Date(data.timestamp).toLocaleString()})`;
  } catch (e) {
    console.error("Hono Error:", e);
    backendMsg.value = "Honoサーバーに接続できません。CORS設定やサーバーの起動を確認してください。";
  } finally {
    isBackendLoading.value = false;
  }
}
</script>

<template>
  <div class="container">
    <header>
      <h1>100pro2026 <span class="badge">Monorepo Active</span></h1>
    </header>

    <main>
      <section class="card">
      <h2>1. Desktop Bridge (Rust + SQLite)</h2>
      <p class="description">Verify the connection to your local tasks.db.</p>
      
      <div class="input-group">
        <button @click="checkDatabase">Check Database Status</button>
      </div>

      <div class="response-box" :class="{ hasValue: greetMsg }">
        {{ greetMsg || "Click the button to ping the DB" }}
      </div>
      </section>

      <section class="card debug-section">
        <h2>🛠 Development Tools</h2>
        <div class="input-group">
          <button @click="seedDatabase">Seed Data</button>
          <button @click="resetDatabase" class="btn-danger">Reset DB</button>
        </div>
      </section>

      <section class="card" v-if="items.length > 0">
        <h2>📋 Active Tasks</h2>
        <div class="task-container">
          <div v-for="item in items" :key="item.id" class="task-row">
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
        <p class="description">localhost:3000 で動作中のHonoからデータを取得します。</p>
        <button @click="fetchFromHono" :disabled="isBackendLoading">
          {{ isBackendLoading ? "通信中..." : "Honoから取得" }}
        </button>
        <div class="response-box" :class="{ hasValue: backendMsg }">
          {{ backendMsg || "ここにHonoからの返答が出ます" }}
        </div>
      </section>
    </main>
  </div>
</template>

<style scoped>
.container {
  max-width: 600px;
  margin: 0 auto;
  padding: 2rem;
  font-family: 'Inter', system-ui, sans-serif;
  color: #2c3e50;
}

header {
  text-align: center;
  margin-bottom: 3rem;
}

.badge {
  font-size: 0.8rem;
  background: #42b883;
  color: white;
  padding: 0.2rem 0.6rem;
  border-radius: 12px;
  vertical-align: middle;
}

.card {
  background: #f8f9fa;
  border-radius: 12px;
  padding: 1.5rem;
  margin-bottom: 2rem;
  box-shadow: 0 4px 6px rgba(0,0,0,0.05);
}

h2 {
  margin-top: 0;
  font-size: 1.25rem;
  color: #35495e;
}

.description {
  font-size: 0.9rem;
  color: #666;
  margin-bottom: 1rem;
}

.input-group {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 1rem;
}

input {
  flex: 1;
  padding: 0.6rem;
  border: 1px solid #ddd;
  border-radius: 6px;
}

button {
  background: #34495e;
  color: white;
  border: none;
  padding: 0.6rem 1.2rem;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.2s;
}

button:hover:not(:disabled) {
  background: #41b883;
}

button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.response-box {
  min-height: 3rem;
  background: #fff;
  border: 1px dashed #ccc;
  border-radius: 6px;
  padding: 1rem;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.95rem;
  color: #999;
}

.response-box.hasValue {
  border-style: solid;
  border-color: #41b883;
  color: #2c3e50;
  background: #f0fff4;
}
.task-container {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-top: 1rem;
}

.task-row {
  display: flex;
  align-items: center;
  background: white;
  padding: 12px;
  border-radius: 8px;
  border: 1px solid #e0e0e0;
  text-align: left;
  transition: transform 0.1s;
}

.task-row:hover {
  transform: translateX(5px);
  border-color: #41b883;
}

.task-info {
  flex: 1;
  margin-left: 15px;
}

.task-info p {
  margin: 4px 0 0 0;
  font-size: 0.85rem;
  color: #666;
}

.status-pill {
  font-size: 0.7rem;
  font-weight: bold;
  padding: 4px 8px;
  border-radius: 4px;
  min-width: 80px;
  text-align: center;
}

.todo { background: #e3f2fd; color: #1976d2; }
.inprogress { background: #fff3e0; color: #f57c00; }
.done { background: #e8f5e9; color: #388e3c; }
.backlog { background: #f5f5f5; color: #616161; }

.motivation {
  font-weight: bold;
  color: #e53935;
}

</style>