<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import type { Item } from '@/types/item';
import TaskList from '@/components/TaskList.vue';

type DrawerMode = 'create' | 'view' | 'edit';

type CreateItemPayload = {
  title: string;
  description: string | null;
  motivation: number | null;
  due: string;
  durationMinutes?: number | null;
};

const props = defineProps<{
  open: boolean;
  mode: DrawerMode;
  selectedItem: Item | null;
  items: Item[];
  syncMap: Record<string, 'pending' | 'success' | 'error' | undefined>;
  errorMap: Record<string, string | undefined>;
  isSyncing: boolean;
  isCreating: boolean;
}>();

const emit = defineEmits<{
  (event: 'update:open', value: boolean): void;
  (event: 'update:mode', value: DrawerMode): void;
  (event: 'select-item', item: Item): void;
  (event: 'create-item', payload: CreateItemPayload): void;
}>();

const createTitle = ref('');
const createDescription = ref<string | null>(null);
const createDue = ref('');
const createDuration = ref<number | null>(null);
const createMotivation = ref(5);

const editTitle = ref('');
const editDescription = ref<string | null>(null);
const editDue = ref('');
const editDuration = ref<number | null>(null);
const editMotivation = ref(5);

const hasSelectedItem = computed(() => props.selectedItem !== null);
const strictSyncMap = computed<Record<string, 'pending' | 'success' | 'error'>>(() => {
  const normalized: Record<string, 'pending' | 'success' | 'error'> = {};
  Object.entries(props.syncMap).forEach(([key, value]) => {
    if (value) {
      normalized[key] = value;
    }
  });
  return normalized;
});

const strictErrorMap = computed<Record<string, string>>(() => {
  const normalized: Record<string, string> = {};
  Object.entries(props.errorMap).forEach(([key, value]) => {
    if (typeof value === 'string') {
      normalized[key] = value;
    }
  });
  return normalized;
});

function toDatetimeLocal(isoValue: string): string {
  const date = new Date(isoValue);
  if (Number.isNaN(date.getTime())) return '';
  date.setMinutes(date.getMinutes() - date.getTimezoneOffset());
  return date.toISOString().slice(0, 16);
}

function resetCreateForm(): void {
  createTitle.value = '';
  createDescription.value = null;
  createDuration.value = null;
  createMotivation.value = 5;

  const now = new Date();
  now.setMinutes(now.getMinutes() - now.getTimezoneOffset());
  createDue.value = now.toISOString().slice(0, 16);
}

function hydrateEditForm(item: Item | null): void {
  if (!item) {
    editTitle.value = '';
    editDescription.value = null;
    editDue.value = '';
    editDuration.value = null;
    editMotivation.value = 5;
    return;
  }

  editTitle.value = item.title;
  editDescription.value = item.description;
  editDue.value = toDatetimeLocal(item.due);
  editDuration.value = item.duration_minutes;
  editMotivation.value = typeof item.motivation === 'number' ? item.motivation : 5;
}

function setMode(mode: DrawerMode): void {
  emit('update:mode', mode);
}

function closeDrawer(): void {
  emit('update:open', false);
}

function submitCreate(): void {
  if (!createTitle.value.trim() || !createDue.value.trim()) return;

  emit('create-item', {
    title: createTitle.value.trim(),
    description: createDescription.value,
    motivation: createMotivation.value,
    due: new Date(createDue.value).toISOString(),
    durationMinutes: createDuration.value,
  });
}

watch(
  () => props.mode,
  (mode) => {
    if (mode === 'create') {
      resetCreateForm();
    }
  },
  { immediate: true },
);

watch(
  () => props.selectedItem,
  (item) => {
    hydrateEditForm(item);
  },
  { immediate: true },
);
</script>

<template>
  <transition
    enter-active-class="transition duration-300 ease-out"
    enter-from-class="translate-x-full opacity-0"
    enter-to-class="translate-x-0 opacity-100"
    leave-active-class="transition duration-200 ease-in"
    leave-from-class="translate-x-0 opacity-100"
    leave-to-class="translate-x-full opacity-0"
  >
    <div v-if="open" class="fixed inset-0 z-40">
      <div class="absolute inset-0 bg-slate-950/25" @click="closeDrawer" />

      <aside class="absolute right-0 top-0 flex h-full w-full max-w-xl flex-col border-l border-slate-200 bg-white shadow-2xl">
        <header class="flex items-center justify-between border-b border-slate-200 px-5 py-4">
          <div>
            <h2 class="text-lg font-semibold text-slate-900">
              {{ mode === 'create' ? 'Create Task' : mode === 'view' ? 'Task Details' : 'Edit Task' }}
            </h2>
            <p class="text-sm text-slate-500">Manage tasks without leaving the plot.</p>
          </div>
          <button
            type="button"
            class="rounded-lg border border-slate-300 px-3 py-2 text-sm text-slate-700 hover:bg-slate-100"
            @click="closeDrawer"
          >
            Close
          </button>
        </header>

        <nav class="flex gap-2 border-b border-slate-200 px-5 py-3">
          <button
            type="button"
            class="rounded-lg px-3 py-2 text-sm font-medium"
            :class="mode === 'view' ? 'bg-slate-900 text-white' : 'bg-slate-100 text-slate-700 hover:bg-slate-200'"
            @click="setMode('view')"
          >
            View
          </button>
          <button
            type="button"
            class="rounded-lg px-3 py-2 text-sm font-medium"
            :class="mode === 'create' ? 'bg-slate-900 text-white' : 'bg-slate-100 text-slate-700 hover:bg-slate-200'"
            @click="setMode('create')"
          >
            Create
          </button>
          <button
            type="button"
            class="rounded-lg px-3 py-2 text-sm font-medium"
            :class="mode === 'edit' ? 'bg-slate-900 text-white' : 'bg-slate-100 text-slate-700 hover:bg-slate-200'"
            @click="setMode('edit')"
            :disabled="!hasSelectedItem"
          >
            Edit
          </button>
        </nav>

        <div class="flex-1 overflow-y-auto p-5">
          <transition
            mode="out-in"
            enter-active-class="transition duration-200 ease-out"
            enter-from-class="translate-y-2 opacity-0"
            enter-to-class="translate-y-0 opacity-100"
            leave-active-class="transition duration-150 ease-in"
            leave-from-class="translate-y-0 opacity-100"
            leave-to-class="translate-y-2 opacity-0"
          >
            <form v-if="mode === 'create'" key="create" @submit.prevent="submitCreate" class="space-y-4">
              <div>
                <label for="create-title" class="mb-1 block text-sm font-medium text-slate-700">Title *</label>
                <input
                  id="create-title"
                  v-model="createTitle"
                  type="text"
                  placeholder="Enter task title"
                  :disabled="isCreating"
                  class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none"
                />
              </div>

              <div>
                <label for="create-description" class="mb-1 block text-sm font-medium text-slate-700">Description</label>
                <textarea
                  id="create-description"
                  v-model="createDescription"
                  rows="3"
                  placeholder="Optional"
                  :disabled="isCreating"
                  class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none"
                />
              </div>

              <div>
                <label for="create-due" class="mb-1 block text-sm font-medium text-slate-700">Due *</label>
                <input
                  id="create-due"
                  v-model="createDue"
                  type="datetime-local"
                  :disabled="isCreating"
                  class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none"
                />
              </div>

              <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
                <div>
                  <label for="create-duration" class="mb-1 block text-sm font-medium text-slate-700">Duration (min)</label>
                  <input
                    id="create-duration"
                    v-model.number="createDuration"
                    type="number"
                    min="1"
                    placeholder="Optional"
                    :disabled="isCreating"
                    class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none"
                  />
                </div>

                <div>
                  <label for="create-motivation" class="mb-1 block text-sm font-medium text-slate-700">Motivation</label>
                  <select
                    id="create-motivation"
                    v-model.number="createMotivation"
                    :disabled="isCreating"
                    class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none"
                  >
                    <option v-for="n in 10" :key="n" :value="n">{{ n }}</option>
                  </select>
                </div>
              </div>

              <div class="flex justify-end gap-2 pt-2">
                <button
                  type="button"
                  class="rounded-lg border border-slate-300 px-4 py-2 text-sm text-slate-700 hover:bg-slate-100"
                  @click="setMode('view')"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  :disabled="!createTitle.trim() || !createDue.trim() || isCreating"
                  class="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50"
                >
                  {{ isCreating ? 'Creating...' : 'Create Task' }}
                </button>
              </div>
            </form>

            <div v-else-if="mode === 'view'" key="view" class="space-y-4">
              <div v-if="selectedItem" class="rounded-xl border border-slate-200 bg-slate-50 p-4">
                <h3 class="text-base font-semibold text-slate-900">{{ selectedItem.title }}</h3>
                <p class="mt-1 text-sm text-slate-600">{{ selectedItem.description || 'No description' }}</p>
                <div class="mt-3 grid grid-cols-2 gap-3 text-xs text-slate-600">
                  <div>
                    <span class="font-medium text-slate-800">Status</span>
                    <p class="mt-1">{{ selectedItem.status }}</p>
                  </div>
                  <div>
                    <span class="font-medium text-slate-800">Due</span>
                    <p class="mt-1">{{ new Date(selectedItem.due).toLocaleString() }}</p>
                  </div>
                  <div>
                    <span class="font-medium text-slate-800">Motivation</span>
                    <p class="mt-1">{{ selectedItem.motivation ?? 'None' }}</p>
                  </div>
                  <div>
                    <span class="font-medium text-slate-800">Duration</span>
                    <p class="mt-1">{{ selectedItem.duration_minutes ?? 'None' }}</p>
                  </div>
                </div>
              </div>

              <TaskList :items="items" :sync-map="strictSyncMap" :error-map="strictErrorMap" :is-syncing="isSyncing" />
            </div>

            <form v-else key="edit" class="space-y-4">
              <div>
                <label for="edit-title" class="mb-1 block text-sm font-medium text-slate-700">Title</label>
                <input
                  id="edit-title"
                  v-model="editTitle"
                  type="text"
                  :disabled="!selectedItem"
                  class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none"
                />
              </div>

              <div>
                <label for="edit-description" class="mb-1 block text-sm font-medium text-slate-700">Description</label>
                <textarea
                  id="edit-description"
                  v-model="editDescription"
                  rows="3"
                  :disabled="!selectedItem"
                  class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none"
                />
              </div>

              <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
                <div>
                  <label for="edit-due" class="mb-1 block text-sm font-medium text-slate-700">Due</label>
                  <input
                    id="edit-due"
                    v-model="editDue"
                    type="datetime-local"
                    :disabled="!selectedItem"
                    class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none"
                  />
                </div>
                <div>
                  <label for="edit-duration" class="mb-1 block text-sm font-medium text-slate-700">Duration</label>
                  <input
                    id="edit-duration"
                    v-model.number="editDuration"
                    type="number"
                    min="1"
                    :disabled="!selectedItem"
                    class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none"
                  />
                </div>
              </div>

              <div>
                <label for="edit-motivation" class="mb-1 block text-sm font-medium text-slate-700">Motivation</label>
                <select
                  id="edit-motivation"
                  v-model.number="editMotivation"
                  :disabled="!selectedItem"
                  class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none"
                >
                  <option v-for="n in 10" :key="n" :value="n">{{ n }}</option>
                </select>
              </div>

              <p class="rounded-lg border border-amber-300 bg-amber-50 px-3 py-2 text-xs text-amber-800">
                Edit mode UI is active and synchronized with selected task. Persisting edits can be wired next to repository update endpoints.
              </p>
            </form>
          </transition>
        </div>
      </aside>
    </div>
  </transition>
</template>
