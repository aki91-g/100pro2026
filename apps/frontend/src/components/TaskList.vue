<script setup lang="ts">
import type { Item } from "@/types/item";
import SyncStatusBadge from "@/components/SyncStatusBadge.vue";
import { useSettings } from "@/composables/useSettings";

const { t } = useSettings();

defineProps<{
  items: Item[];
  syncMap: Record<string, "pending" | "success" | "error">;
  errorMap: Record<string, string>;
  isSyncing: boolean;
}>();

const emit = defineEmits<{
  (event: 'select-item', item: Item): void;
}>();

function displayStatus(status: Item['status']): string {
  if (status === 'backlog') return t('statusBacklog');
  if (status === 'inprogress') return t('statusDoing');
  if (status === 'done') return t('statusDone');
  return t('statusTodo');
}
</script>

<template>
  <section v-if="items.length > 0" class="task-section">
    <header class="section-header">
      <span class="icon">📋</span>
      <h2>{{ t('list') }}</h2>
      <span class="count">{{ items.length }}</span>
    </header>

    <div class="task-container">
      <button
        v-for="item in items"
        :key="item.id"
        type="button"
        class="task-row"
        @click="emit('select-item', item)"
      >
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
              {{ displayStatus(item.status) }}
            </span>
            <span class="motivation">
              <span class="fire">🔥</span> {{ item.motivation }}
            </span>
          </div>

          <div class="task-content">
            <strong class="task-title">{{ item.title }}</strong>
            <p v-if="item.description" class="task-desc">{{ item.description }}</p>
          </div>
        </div>

        <div class="task-chevron">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
          </svg>
        </div>
      </button>
    </div>
  </section>
</template>

<style scoped>
/* --- Section Container --- */
.task-section {
  padding: 0.5rem 0;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  margin-bottom: 1.25rem;
  padding-left: 0.5rem;
}

.section-header .icon {
  font-size: 1.2rem;
}

.section-header h2 {
  font-size: 1rem;
  font-weight: 800;
  color: var(--text-strong);
  letter-spacing: 0.05em;
  text-transform: uppercase;
  margin: 0;
}

.section-header .count {
  background: rgba(168, 85, 247, 0.1);
  color: #a855f7;
  font-size: 0.75rem;
  font-weight: 700;
  padding: 0.1rem 0.6rem;
  border-radius: 20px;
}

/* --- List Layout --- */
.task-container {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

/* --- Task Row (Individual Card) --- */
.task-row {
  display: flex;
  align-items: center;
  width: 100%;
  text-align: left;
  padding: 1rem 1.25rem;
  background: color-mix(in srgb, var(--bg-primary) 72%, transparent);
  border: 1px solid color-mix(in srgb, var(--tg-border-default) 82%, transparent);
  border-radius: 16px;
  cursor: pointer;
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

.task-row:active {
  transform: translateY(0);
}

/* --- Status & Badge --- */
.status-indicator {
  margin-right: 1rem;
  display: flex;
  align-items: center;
}

.task-main {
  flex: 1;
  min-width: 0; /* テキスト溢れ防止 */
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

/* 各ステータスの色味をグラスデザインに合わせ調整 */
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

/* --- Text Content --- */
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

/* --- Decorative Chevron --- */
.task-chevron {
  margin-left: 1rem;
  color: var(--tg-border-default);
  transition: transform 0.3s ease, color 0.3s ease;
}

.task-row:hover .task-chevron {
  color: #a855f7;
  transform: translateX(3px);
}
</style>