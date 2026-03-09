<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

// Composables
import { useAuth } from '@/composables/useAuth';
import { useItems } from '@/composables/useItems';
import { useSyncStatus } from '@/composables/useSyncStatus';

// Components
import TaskList from '@/components/TaskList.vue';

// Auth
const { userId } = useAuth();

// Items with session management
const {
  items,
  isSyncing,
  fetchActiveItems,
  createItem,
  startNewSession,
  getCurrentToken,
} = useItems();

// Sync status
const { syncMap, errorMap } = useSyncStatus();

// Local state
const greetMsg = ref('');
let unlistenRemoteCatchup: UnlistenFn | null = null;

// New Item Form state
const newItemTitle = ref('');
const newItemDue = ref('');
const newItemDuration = ref<number | null>(null);
const newItemMotivation = ref<number>(5);
const isCreating = ref(false);

type UUID = string;

interface RemoteCatchupEvent {
  user_id: UUID;
  synced_count: number;
}

// --- Lifecycle ---
onMounted(async () => {
  // Initialize due with a valid datetime-local value for better UX.
  const now = new Date();
  now.setMinutes(now.getMinutes() - now.getTimezoneOffset());
  newItemDue.value = now.toISOString().slice(0, 16);

  unlistenRemoteCatchup = await listen<RemoteCatchupEvent>('remote-catchup', (event) => {
    const { user_id, synced_count } = event.payload;
    if (userId.value && user_id === userId.value) {
      greetMsg.value = `Remote connected. Synced ${synced_count} local tasks to cloud.`;
    }
  });

  // Load items on mount with a new session token
  const sessionToken = startNewSession();
  await loadItems(sessionToken);
});

onUnmounted(() => {
  if (unlistenRemoteCatchup) {
    unlistenRemoteCatchup();
    unlistenRemoteCatchup = null;
  }
});

// Watch sync map for real-time updates
watch(
  syncMap,
  (nextMap) => {
    for (const item of items.value) {
      const eventStatus = nextMap[item.id];

      if (eventStatus === 'pending' && item.sync_status !== 'local_only') {
        item.sync_status = 'modified';
      }

      if (eventStatus === 'success') {
        item.sync_status = 'synced';
      }

      if (eventStatus === 'error' && item.sync_status === 'synced') {
        item.sync_status = 'modified';
      }
    }
  },
  { deep: true }
);

// --- Item Management ---
async function loadItems(sessionToken: number) {
  try {
    await fetchActiveItems(sessionToken);
    // Only update message if this session is still valid
    if (sessionToken === getCurrentToken()) {
      greetMsg.value = `Connected! Showing ${items.value.length} tasks.`;
    }
  } catch (e) {
    // Only update error if this session is still valid
    if (sessionToken === getCurrentToken()) {
      console.error('Fetch Error:', e);
      greetMsg.value = 'Failed to load tasks.';
    }
  }
}

async function handleRefreshItems() {
  await loadItems(getCurrentToken());
}

// --- Create New Item ---
async function handleCreateItem() {
  if (!newItemTitle.value.trim() || !newItemDue.value.trim()) {
    return;
  }

  const dueIso = new Date(newItemDue.value).toISOString();

  isCreating.value = true;
  try {
    await createItem(
      newItemTitle.value.trim(),
      newItemMotivation.value,
      dueIso,
      newItemDuration.value
    );
    // Clear form after successful creation
    newItemTitle.value = '';
    const now = new Date();
    now.setMinutes(now.getMinutes() - now.getTimezoneOffset());
    newItemDue.value = now.toISOString().slice(0, 16);
    newItemDuration.value = null;
    newItemMotivation.value = 5;
    greetMsg.value = `✓ Item created successfully! Showing ${items.value.length} tasks.`;
  } catch (e) {
    console.error('Create Item Error:', e);
    greetMsg.value = 'Failed to create item.';
  } finally {
    isCreating.value = false;
  }
}
</script>

<template>
  <div class="container">
    <header>
      <div class="sync-section">
        <span v-if="isSyncing" class="syncing-indicator">Syncing in background...</span>
      </div>
    </header>

    <main>
      <section class="card">
        <h2>1. Desktop Bridge (Rust + SQLite)</h2>
        <p class="description">Current connection to your local tasks.db.</p>
        <div class="input-group">
          <button @click="handleRefreshItems">Refresh List</button>
        </div>
        <div class="response-box" :class="{ hasValue: greetMsg }">
          {{ greetMsg || 'Status unknown' }}
        </div>
      </section>

      <section class="card">
        <h2>Add New Item</h2>
        <p class="description">Create a new task to add to your list.</p>
        <form @submit.prevent="handleCreateItem" class="new-item-form">
          <div class="form-row">
            <div class="form-field">
              <label for="item-title">Title *</label>
              <input 
                id="item-title"
                v-model="newItemTitle"
                type="text" 
                placeholder="Enter task title"
                :disabled="isCreating"
              />
            </div>
            <div class="form-field">
              <label for="item-due">Due *</label>
              <input
                id="item-due"
                v-model="newItemDue"
                type="datetime-local"
                :disabled="isCreating"
              />
            </div>
          </div>
          <div class="form-row">
            <div class="form-field">
              <label for="item-duration">Duration (minutes)</label>
              <input 
                id="item-duration"
                v-model.number="newItemDuration"
                type="number" 
                placeholder="Optional"
                :disabled="isCreating"
                min="1"
              />
            </div>
            <div class="form-field">
              <label for="item-motivation">Motivation (1-10)</label>
              <select 
                id="item-motivation"
                v-model.number="newItemMotivation"
                :disabled="isCreating"
              >
                <option v-for="n in 10" :key="n" :value="n">{{ n }}</option>
              </select>
            </div>
          </div>
          <button 
            type="submit" 
            :disabled="!newItemTitle.trim() || !newItemDue.trim() || isCreating"
            class="create-button"
          >
            {{ isCreating ? 'Creating...' : 'Create Item' }}
          </button>
        </form>
      </section>

      <TaskList :items="items" :sync-map="syncMap" :error-map="errorMap" :is-syncing="isSyncing" />
    </main>
  </div>
</template>

<style scoped>
.container {
  max-width: 700px;
  margin: 0 auto;
  padding: 2rem;
  font-family: 'Inter', sans-serif;
}

header {
  margin-bottom: 2rem;
}
.sync-section {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 1rem;
}

.syncing-indicator {
  font-size: 0.85rem;
  color: #3498db;
}

.card {
  background: #f8f9fa;
  border-radius: 12px;
  padding: 1.5rem;
  margin-bottom: 1.5rem;
  border: 1px solid #eee;
}
.description {
  color: #666;
  font-size: 0.9rem;
  margin-bottom: 1rem;
}

.input-group {
  display: flex;
  gap: 10px;
  margin-bottom: 1rem;
}
button {
  background: #34495e;
  color: white;
  border: none;
  padding: 0.6rem 1.2rem;
  border-radius: 6px;
  cursor: pointer;
}
button:hover {
  background: #41b883;
}
button:disabled {
  background: #999;
  cursor: not-allowed;
}

.response-box {
  min-height: 2.5rem;
  background: #fff;
  border: 1px dashed #ccc;
  border-radius: 6px;
  padding: 0.8rem;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.9rem;
  color: #666;
}
.response-box.hasValue {
  border-style: solid;
  border-color: #42b883;
  color: #2c3e50;
  background: #f0fff4;
}

/* New Item Form Styles */
.new-item-form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.form-row {
  display: flex;
  gap: 1rem;
  flex-wrap: wrap;
}

.form-field {
  flex: 1;
  min-width: 200px;
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.form-field label {
  font-size: 0.85rem;
  font-weight: 500;
  color: #555;
}

.form-field input,
.form-field select {
  padding: 0.7rem;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 0.95rem;
  font-family: inherit;
  background: #fff;
}

.form-field input:focus,
.form-field select:focus {
  outline: none;
  border-color: #41b883;
  box-shadow: 0 0 0 3px rgba(65, 184, 131, 0.1);
}

.form-field input:disabled,
.form-field select:disabled {
  background: #f5f5f5;
  cursor: not-allowed;
}

.create-button {
  align-self: flex-start;
  background: #41b883;
  color: white;
  border: none;
  padding: 0.8rem 1.8rem;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 500;
  transition: background 0.2s;
}

.create-button:hover:not(:disabled) {
  background: #35a379;
}

.create-button:disabled {
  background: #999;
  cursor: not-allowed;
}

</style>
