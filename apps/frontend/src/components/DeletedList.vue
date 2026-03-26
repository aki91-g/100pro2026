<script setup lang="ts">
import type { Item } from "@/types/item";
import ActionableItemRow from "@/components/ActionableItemRow.vue";
import { useSettings } from "@/composables/useSettings";

const { t } = useSettings();

defineProps<{
  items: Item[];
  syncMap: Record<string, "pending" | "success" | "error">;
  errorMap: Record<string, string>;
  isSyncing: boolean;
  isProcessing?: boolean;
}>();

const emit = defineEmits<{
  (event: 'restore-item', item: Item): void;
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
    <div class="task-container">
      <ActionableItemRow
        v-for="item in items"
        :key="item.id"
        :item="item"
        :sync-map="syncMap"
        :error-map="errorMap"
        :is-syncing="isSyncing"
        :is-processing="isProcessing"
        :action-label="t('restore')"
        :motivation-fallback="t('drawerNoMotivationValue')"
        :status-label="displayStatus"
        @action="emit('restore-item', item)"
      />
    </div>
  </section>
</template>

<style scoped>
/* --- Section Container --- */
.task-section {
  padding: 0.5rem 0;
}

/* --- List Layout --- */
.task-container {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}
</style>