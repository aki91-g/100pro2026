<script setup lang="ts">
import type { Item } from '@/types/item';
import SyncStatusBadge from '@/components/SyncStatusBadge.vue';

defineProps<{
  item: Item;
  syncMap: Record<string, 'pending' | 'success' | 'error'>;
  errorMap: Record<string, string>;
  isSyncing: boolean;
  isProcessing?: boolean;
  actionLabel: string;
  statusLabel: (status: Item['status']) => string;
}>();

const emit = defineEmits<{
  (event: 'action', item: Item): void;
}>();
</script>

<template>
  <div class="task-row">
    <div class="status-indicator">
      <SyncStatusBadge
        :sync-status="item.sync_status"
        :event-status="syncMap[item.id]"
        :error-message="errorMap[item.id]"
        :is-syncing="isSyncing"
      />
    </div>

    <div class="task-main">
      <div class="task-top">
        <span :class="['status-pill', item.status.toLowerCase()]">
          {{ statusLabel(item.status) }}
        </span>
        <span class="motivation">
          <span class="fire">🔥</span> {{ item.motivation ?? 'N/A' }}
        </span>
      </div>

      <div class="task-content">
        <strong class="task-title">{{ item.title }}</strong>
        <p v-if="item.description" class="task-desc">{{ item.description }}</p>
      </div>
    </div>

    <div class="task-action">
      <button
        type="button"
        class="row-action"
        :disabled="isProcessing"
        @click="emit('action', item)"
      >
        {{ actionLabel }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.task-row {
  display: flex;
  align-items: center;
  width: 100%;
  text-align: left;
  padding: 1rem 1.25rem;
  background: color-mix(in srgb, var(--bg-primary) 72%, transparent);
  border: 1px solid color-mix(in srgb, var(--tg-border-default) 82%, transparent);
  border-radius: 16px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  overflow: hidden;
}

.task-row:hover {
  background: var(--bg-primary);
  border-color: #a855f7;
  transform: translateY(-2px);
  box-shadow: 0 10px 20px rgba(0, 0, 0, 0.04);
}

.status-indicator {
  margin-right: 1rem;
  display: flex;
  align-items: center;
}

.task-main {
  flex: 1;
  min-width: 0;
}

.task-top {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin-bottom: 0.4rem;
}

.status-pill {
  font-size: 0.65rem;
  font-weight: 800;
  padding: 0.2rem 0.6rem;
  border-radius: 8px;
  text-transform: uppercase;
  letter-spacing: 0.02em;
}

.todo { background: var(--status-bg-todo); color: var(--status-text-todo); }
.inprogress { background: var(--status-bg-inprogress); color: var(--status-text-inprogress); }
.done { background: var(--status-bg-done); color: var(--status-text-done); }
.backlog {
  background: var(--status-bg-backlog);
  color: var(--status-text-backlog);
  border: 1px dashed var(--status-border-backlog);
}

.motivation {
  font-size: 0.75rem;
  font-weight: 700;
  color: #ef4444;
  display: flex;
  align-items: center;
  gap: 0.2rem;
}

.fire {
  font-size: 0.85rem;
}

.task-content {
  display: flex;
  flex-direction: column;
}

.task-title {
  font-size: 0.95rem;
  font-weight: 700;
  color: var(--text-strong);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.task-desc {
  font-size: 0.8rem;
  color: var(--text-muted);
  margin-top: 0.1rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.task-action {
  margin-left: 1rem;
  display: flex;
  align-items: center;
}

.row-action {
  border: 1px solid var(--tg-border-default);
  background: var(--bg-primary);
  color: var(--text-primary);
  border-radius: 0.65rem;
  padding: 0.45rem 0.7rem;
  font-size: 0.78rem;
  font-weight: 700;
  cursor: pointer;
  transition: border-color 0.2s ease, background-color 0.2s ease;
}

.row-action:hover {
  border-color: #a855f7;
  background: color-mix(in srgb, var(--bg-primary) 70%, #ffffff);
}

.row-action:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>