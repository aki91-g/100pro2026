<script setup lang="ts">
import type { Item } from "@/services/itemService";

defineProps<{
  items: Item[];
  syncMap: Record<string, "pending" | "success" | "error">;
  errorMap: Record<string, string>;
}>();
</script>

<template>
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
</template>

<style scoped>
.card { background: #f8f9fa; border-radius: 12px; padding: 1.5rem; margin-bottom: 1.5rem; border: 1px solid #eee; }
.task-container { display: flex; flex-direction: column; gap: 8px; margin-top: 1rem; }
.task-row { display: flex; align-items: center; background: white; padding: 12px; border-radius: 8px; border: 1px solid #e0e0e0; }
.task-info { flex: 1; margin-left: 12px; text-align: left; }
.task-info p { margin: 2px 0 0 0; font-size: 0.8rem; color: #777; }
.task-meta { display: flex; align-items: center; gap: 8px; }
.status-pill { font-size: 0.65rem; font-weight: bold; padding: 3px 6px; border-radius: 4px; min-width: 70px; text-align: center; }

.todo { background: #e3f2fd; color: #1976d2; }
.inprogress { background: #fff3e0; color: #f57c00; }
.done { background: #e8f5e9; color: #388e3c; }
.backlog { background: #f5f5f5; color: #616161; }
.motivation { color: #e53935; font-weight: bold; font-size: 0.85rem; }

.sync-tag { margin-right: 8px; font-size: 0.8rem; }
.dot.pending { color: #3498db; animation: blink 1s infinite; }
.dot.success { color: #42b883; font-weight: bold; }
.dot.error { color: #e53935; font-weight: bold; cursor: help; }
@keyframes blink {
  50% { opacity: 0; }
}
</style>