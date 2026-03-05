<script setup lang="ts">
interface Props {
  syncStatus: 'synced' | 'local_only' | 'modified';
  eventStatus?: 'pending' | 'success' | 'error';
  errorMessage?: string;
  isSyncing?: boolean;
}

const props = defineProps<Props>();

function getLabel() {
  if (props.eventStatus === 'pending') return 'syncing';
  if (props.eventStatus === 'error') return 'modified';
  if (props.eventStatus === 'success') return 'synced';
  if (props.syncStatus === 'local_only') return 'local only';
  return props.syncStatus;
}
</script>

<template>
  <span
    :class="[
      'sync-status-badge',
      syncStatus,
      { pending: eventStatus === 'pending', syncing: isSyncing && syncStatus !== 'synced' }
    ]"
    :title="eventStatus === 'error' ? errorMessage : undefined"
  >
    {{ getLabel() }}
  </span>
</template>

<style scoped>
.sync-status-badge {
  position: absolute;
  top: 8px;
  right: 8px;
  font-size: 0.65rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.02em;
  padding: 3px 7px;
  border-radius: 999px;
}

.synced {
  background: #e8f5e9;
  color: #2e7d32;
}

.local_only {
  background: #e3f2fd;
  color: #1565c0;
}

.modified {
  background: #fff3e0;
  color: #ef6c00;
}

.pending {
  background: #ede7f6;
  color: #5e35b1;
}

.syncing {
  animation: pulse 1.2s ease-in-out infinite;
}

@keyframes pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.6;
  }
}
</style>
