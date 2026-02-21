<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

// --- 状態管理 ---
const name = ref("");
const greetMsg = ref("");
const backendMsg = ref("");
const isBackendLoading = ref(false);

// --- ロジック: Rust (Desktop) との通信 ---
async function greet() {
  if (!name.value) {
    greetMsg.value = "名前を入力してください";
    return;
  }
  try {
    // Rustの 'greet' コマンドを呼び出し
    greetMsg.value = await invoke("greet", { name: name.value });
  } catch (e) {
    console.error("Rust Error:", e);
    greetMsg.value = "Rustとの通信に失敗しました";
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
    backendMsg.value = `${data.message} (取得時刻: ${new Date(data.timestamp).toLocaleTimeString()})`;
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
        <h2>1. Desktop Bridge (Rust)</h2>
        <p class="description">Tauri経由でRustのネイティブ機能を呼び出します。</p>
        <div class="input-group">
          <input v-model="name" placeholder="名前を入力..." @keyup.enter="greet" />
          <button @click="greet">Rustを呼ぶ</button>
        </div>
        <div class="response-box" :class="{ hasValue: greetMsg }">
          {{ greetMsg || "ここにRustからの返答が出ます" }}
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
</style>