import { ref, onMounted, onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

interface SyncEvent {
  id: string;
  status: 'pending' | 'success' | 'error';
  message: string | null;
}

export const useSyncStatus = () => {
  const syncMap = ref<Record<string, SyncEvent['status']>>({});
  const errorMap = ref<Record<string, string>>({});
  let unlisten: UnlistenFn | null = null;
  let disposed = false;

  onMounted(async () => {
    const stop = await listen<SyncEvent>('sync-status', (event) => {
      const { id, status, message } = event.payload;
      syncMap.value[id] = status;

      if (status === 'error' && message) {
        errorMap.value[id] = message;
      } else {
        delete errorMap.value[id];
      }

      // Cleanup success status after a few seconds
      if (status === 'success') {
        setTimeout(() => {
          if (syncMap.value[id] === 'success') delete syncMap.value[id];
        }, 3000);
      }
    });
    if (disposed) {
      stop();
    } else {
      unlisten = stop;
    }
  });

  onUnmounted(() => {
    disposed = true;
    if (unlisten) unlisten();
  });

  return { syncMap, errorMap };
};