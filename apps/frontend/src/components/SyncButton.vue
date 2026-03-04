<script setup lang="ts">
import { ref, computed, onUnmounted } from 'vue';
import { useAuth } from '@/composables/useAuth';
import { useItems } from '@/composables/useItems';

const { isAuthenticated, userId } = useAuth();
const { syncWithRemote, isSyncing } = useItems();

const syncResult = ref<{ count: number; error: string | null }>({ count: 0, error: null });
let errorTimer: ReturnType<typeof setTimeout> | null = null;

const isDisabled = computed(() => !isAuthenticated.value || isSyncing.value);

async function handleSync() {
  if (!isAuthenticated.value) {
    syncResult.value.error = "Please login first";
    return;
  }

  syncResult.value.error = null;
  
  try {
    await syncWithRemote();
    syncResult.value.count++; 
  } catch (err) {
    syncResult.value.error = String(err);
  } finally {
    // Auto-clear error message after 3 seconds
    if (errorTimer) {
      clearTimeout(errorTimer);
    }
    errorTimer = setTimeout(() => { syncResult.value.error = null; }, 3000);
  }
}

onUnmounted(() => {
  if (errorTimer) {
    clearTimeout(errorTimer);
  }
});
</script>

<template>
  <div class="sync-container">
    <button 
      @click="handleSync" 
      :disabled="isDisabled"
      :class="{ 'is-loading': isSyncing }"
      class="sync-btn"
    >
      <span v-if="isSyncing">🔄 Syncing...</span>
      <span v-else-if="!isAuthenticated">🔒 Login to Sync</span>
      <span v-else>🔄 Sync Cloud</span>
    </button>

    <Transition name="fade">
      <p v-if="syncResult.error" class="error-msg">
        ❌ {{ syncResult.error }}
      </p>
    </Transition>
  </div>
</template>

<style scoped>
.sync-container {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.sync-btn {
  padding: 0.6rem 1.2rem;
  background-color: #3b82f6;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 600;
  transition: all 0.2s ease;
}

.sync-btn:hover:not(:disabled) {
  background-color: #2563eb;
}

.sync-btn:disabled {
  background-color: #94a3b8;
  cursor: not-allowed;
}

.is-loading {
  animation: pulse 1.5s infinite;
}

.error-msg {
  font-size: 0.8rem;
  color: #ef4444;
}

@keyframes pulse {
  0% { opacity: 1; }
  50% { opacity: 0.6; }
  100% { opacity: 1; }
}

.fade-enter-active, .fade-leave-active {
  transition: opacity 0.5s;
}
.fade-enter-from, .fade-leave-to {
  opacity: 0;
}
</style>