<script setup lang="ts">
import { ref } from 'vue';
import { syncAndRefresh } from '../services/itemService'; // Adjust path as needed

const isSyncing = ref(false);
const syncResult = ref<{ count: number; error: string | null }>({ count: 0, error: null });

async function handleSync() {
  isSyncing.value = true;
  syncResult.value.error = null;
  
  try {
    // Calling the TS function we drafted earlier
    await syncAndRefresh();
    // In a real app, you might want the count from the invoke call here
    syncResult.value.count++; 
  } catch (err) {
    syncResult.value.error = String(err);
  } finally {
    isSyncing.value = false;
    // Auto-clear the message after 3 seconds
    setTimeout(() => { syncResult.value.error = null; }, 3000);
  }
}
</script>

<template>
  <div class="sync-container">
    <button 
      @click="handleSync" 
      :disabled="isSyncing"
      :class="{ 'is-loading': isSyncing }"
      class="sync-btn"
    >
      <span v-if="isSyncing">🔄 Syncing...</span>
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