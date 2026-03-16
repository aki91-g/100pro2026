<script setup lang="ts">
import type { Item } from "@/types/item";
import SyncStatusBadge from "@/components/SyncStatusBadge.vue";

defineProps<{
  items: Item[];
  syncMap: Record<string, "pending" | "success" | "error">;
  errorMap: Record<string, string>;
  isSyncing: boolean;
}>();

const emit = defineEmits<{
  (event: "select-item", item: Item): void;
  (event: "edit-item", item: Item): void;
}>();

function handleSelectItem(item: Item): void {
  emit("select-item", item);
}

function handleEditItem(item: Item): void {
  emit("edit-item", item);
}
</script>

<template>
  <section class="card" v-if="items.length > 0">
    <h2>📋 Tasks</h2>
    <div class="task-container">
      <div v-for="item in items" :key="item.id" class="task-row" @click="handleSelectItem(item)">
        <SyncStatusBadge
          :sync-status="item.sync_status"
          :event-status="syncMap[item.id]"
          :error-message="errorMap[item.id]"
          :is-syncing="isSyncing"
        />

        <span :class="['status-pill', item.status.toLowerCase()]">
          {{ item.status }}
        </span>

        <div class="task-info">
          <strong>{{ item.title }}</strong>
          <p v-if="item.description">{{ item.description }}</p>
        </div>

        <div class="task-meta">
          <span class="motivation">🔥 {{ item.motivation }}</span>
          <button
            type="button"
            class="edit-button"
            @click.stop="handleEditItem(item)"
          >
            Edit
          </button>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.card { background: #f8f9fa; border-radius: 12px; padding: 1.5rem; margin-bottom: 1.5rem; border: 1px solid #eee; }
.task-container { display: flex; flex-direction: column; gap: 8px; margin-top: 1rem; }
.task-row { position: relative; display: flex; align-items: center; background: white; padding: 16px 12px 12px; border-radius: 8px; border: 1px solid #e0e0e0; }
.task-row { cursor: pointer; }
.task-row:hover { border-color: #cbd5e1; }
.task-info { flex: 1; margin-left: 12px; text-align: left; }
.task-info p { margin: 2px 0 0 0; font-size: 0.8rem; color: #777; }
.task-meta { display: flex; align-items: center; gap: 8px; }
.status-pill { font-size: 0.65rem; font-weight: bold; padding: 3px 6px; border-radius: 4px; min-width: 70px; text-align: center; }

.todo { background: #e3f2fd; color: #1976d2; }
.inprogress { background: #fff3e0; color: #f57c00; }
.done { background: #e8f5e9; color: #388e3c; }
.backlog { background: #f5f5f5; color: #616161; }
.motivation { color: #e53935; font-weight: bold; font-size: 0.85rem; }
.edit-button { border: 1px solid #cbd5e1; background: #fff; color: #0f172a; border-radius: 6px; padding: 4px 8px; font-size: 0.75rem; font-weight: 600; }
.edit-button:hover { background: #f8fafc; }

</style>