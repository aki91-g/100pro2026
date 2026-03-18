<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import type { Item } from '@/types/item';
import TaskList from '@/components/TaskList.vue';
import { useItems } from '@/composables/useItems';

type DrawerMode = 'create' | 'view' | 'edit' | 'tasks';

const props = defineProps<{
  open: boolean;
  mode: DrawerMode;
  selectedItem: Item | null;
  items: Item[];
  syncMap: Record<string, 'pending' | 'success' | 'error' | undefined>;
  errorMap: Record<string, string | undefined>;
  isSyncing: boolean;
}>();

const emit = defineEmits<{
  (event: 'update:open', value: boolean): void;
  (event: 'update:mode', value: DrawerMode): void;
  (event: 'select-item', item: Item): void;
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
const isSavingEdit = ref(false);
const isCreating = ref(false);
const isArchiving = ref(false);
const isDeleting = ref(false);
const previousMode = ref<'view' | 'tasks'>('view');

const { createItem, updateItem, archiveItem, softDeleteItem, items: repositoryItems } = useItems();

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

function goToTasks(): void {
  emit('update:mode', 'tasks');
}

function goToCreate(): void {
  emit('update:mode', 'create');
}

function startEdit(): void {
  previousMode.value = 'view';
  emit('update:mode', 'edit');
}

function cancelEdit(): void {
  emit('update:mode', previousMode.value);
}

function cancelCreate(): void {
  emit('update:mode', 'tasks');
}

function handleTaskListSelect(item: Item): void {
  emit('select-item', item);
  emit('update:mode', 'view');
}

function closeDrawer(): void {
  emit('update:open', false);
}

function handleKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape' && props.open) {
    closeDrawer();
  }
}

async function submitCreate(): Promise<void> {
  if (!createTitle.value.trim() || !createDue.value.trim()) return;

  isCreating.value = true;
  try {
    const id = await createItem({
      title: createTitle.value.trim(),
      description: createDescription.value,
      motivation: createMotivation.value,
      due: new Date(createDue.value).toISOString(),
      durationMinutes: createDuration.value,
    });

    const created = repositoryItems.value.find((item) => item.id === id) ?? null;
    if (created) {
      emit('select-item', created);
      emit('update:mode', 'view');
    } else {
      console.warn('Created item not found in repository:', id);
    }
  } catch (error) {
    console.error('Failed to create item:', error);
  } finally {
    isCreating.value = false;
  }
}

async function handleArchive(): Promise<void> {
  if (!props.selectedItem) return;

  isArchiving.value = true;
  try {
    await archiveItem(props.selectedItem.id);
    emit('update:open', false);
    emit('update:mode', 'view');
  } catch (error) {
    console.error('Failed to archive item:', error);
  } finally {
    isArchiving.value = false;
  }
}

async function handleDelete(): Promise<void> {
  if (!props.selectedItem) return;
  if (!confirm('Are you sure you want to delete this task?')) return;

  isDeleting.value = true;
  try {
    await softDeleteItem(props.selectedItem.id);
    emit('update:open', false);
    emit('update:mode', 'view');
  } catch (error) {
    console.error('Failed to delete item:', error);
  } finally {
    isDeleting.value = false;
  }
}

async function handleEditSubmit(): Promise<void> {
  if (!props.selectedItem || !editTitle.value.trim() || !editDue.value.trim()) {
    return;
  }

  isSavingEdit.value = true;
  try {
    await updateItem({
      id: props.selectedItem.id,
      title: editTitle.value.trim(),
      description: editDescription.value,
      due: new Date(editDue.value).toISOString(),
      durationMinutes: editDuration.value,
      motivation: editMotivation.value,
    });

    emit('select-item', {
      ...props.selectedItem,
      title: editTitle.value.trim(),
      description: editDescription.value,
      due: new Date(editDue.value).toISOString(),
      duration_minutes: editDuration.value,
      motivation: editMotivation.value,
      sync_status: 'modified',
    });
    emit('update:mode', previousMode.value);
  } catch (error) {
    console.error('Failed to save item changes:', error);
  } finally {
    isSavingEdit.value = false;
  }
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

onMounted(() => {
  window.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown);
});
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

      <aside role="dialog" aria-modal="true" class="absolute right-0 top-0 flex h-full w-full max-w-xl flex-col border-l border-slate-200 bg-white shadow-2xl">
        <header class="flex items-center justify-between border-b border-slate-200 px-5 py-4">
          <div class="flex items-center gap-3">
            <!-- Back to List button visible in 'view' mode -->
            <button
              v-if="mode === 'view'"
              type="button"
              class="flex items-center gap-1 rounded-lg border border-slate-300 bg-white px-3 py-1.5 text-sm font-medium text-slate-700 hover:bg-slate-100"
              @click="goToTasks"
              title="Return to task list"
            >
              <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"></path>
              </svg>
              <span>Back to List</span>
            </button>
            <h2 class="text-lg font-semibold text-slate-900">
              {{ mode === 'create' ? 'Create Task' : mode === 'view' ? 'Task Details' : mode === 'edit' ? 'Edit Task' : 'Tasks' }}
            </h2>
          </div>

          <div class="flex items-center gap-2">
            <!-- New Task button visible in 'tasks' mode -->
            <button
              v-if="mode === 'tasks'"
              type="button"
              class="flex items-center gap-1 rounded-lg bg-blue-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-blue-700"
              @click="goToCreate"
              title="Create a new task"
            >
              <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path>
              </svg>
              <span>New Task</span>
            </button>
            <button
              type="button"
              class="rounded-lg border border-slate-300 px-3 py-2 text-sm text-slate-700 hover:bg-slate-100"
              @click="closeDrawer"
            >
              Close
            </button>
          </div>
        </header>

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
                  @click="cancelCreate"
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
              <div v-if="selectedItem" class="relative rounded-xl border border-slate-200 bg-slate-50 p-4">
                <button
                  type="button"
                  class="absolute right-4 top-4 rounded-lg border border-slate-300 bg-white px-3 py-1 text-sm font-medium text-slate-700 hover:bg-slate-100"
                  @click="startEdit"
                  title="Edit this task"
                >
                  Edit
                </button>
                <h3 class="pr-16 text-base font-semibold text-slate-900">{{ selectedItem.title }}</h3>
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
            </div>

            <div v-else-if="mode === 'tasks'" key="tasks" class="space-y-2">
              <TaskList :items="items" :sync-map="strictSyncMap" :error-map="strictErrorMap" :is-syncing="isSyncing" @select-item="handleTaskListSelect" />
            </div>

            <form v-else key="edit" class="space-y-4" @submit.prevent="handleEditSubmit">
              <div>
                <label for="edit-title" class="mb-1 block text-sm font-medium text-slate-700">Title</label>
                <input
                  id="edit-title"
                  v-model="editTitle"
                  type="text"
                  :disabled="!selectedItem || isSavingEdit"
                  class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none"
                />
              </div>

              <div>
                <label for="edit-description" class="mb-1 block text-sm font-medium text-slate-700">Description</label>
                <textarea
                  id="edit-description"
                  v-model="editDescription"
                  rows="3"
                  :disabled="!selectedItem || isSavingEdit"
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
                    :disabled="!selectedItem || isSavingEdit"
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
                    :disabled="!selectedItem || isSavingEdit"
                    class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none"
                  />
                </div>
              </div>

              <div>
                <label for="edit-motivation" class="mb-1 block text-sm font-medium text-slate-700">Motivation</label>
                <select
                  id="edit-motivation"
                  v-model.number="editMotivation"
                  :disabled="!selectedItem || isSavingEdit"
                  class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none"
                >
                  <option v-for="n in 10" :key="n" :value="n">{{ n }}</option>
                </select>
              </div>

              <div class="flex items-center justify-between gap-2">
                <div class="flex gap-2">
                  <button
                    type="button"
                    class="rounded-lg border border-amber-300 bg-amber-50 px-4 py-2 text-sm text-amber-700 hover:bg-amber-100 disabled:cursor-not-allowed disabled:opacity-50"
                    :disabled="!selectedItem || isSavingEdit || isArchiving || isDeleting"
                    @click="handleArchive"
                  >
                    {{ isArchiving ? 'Archiving...' : 'Archive' }}
                  </button>
                  <button
                    type="button"
                    class="rounded-lg border border-red-300 bg-red-50 px-4 py-2 text-sm text-red-700 hover:bg-red-100 disabled:cursor-not-allowed disabled:opacity-50"
                    :disabled="!selectedItem || isSavingEdit || isArchiving || isDeleting"
                    @click="handleDelete"
                  >
                    {{ isDeleting ? 'Deleting...' : 'Delete' }}
                  </button>
                </div>
                <div class="flex gap-2">
                  <button
                    type="button"
                    class="rounded-lg border border-slate-300 px-4 py-2 text-sm text-slate-700 hover:bg-slate-100"
                    :disabled="isSavingEdit || isArchiving || isDeleting"
                    @click="cancelEdit"
                  >
                    Cancel
                  </button>
                  <button
                    type="submit"
                    class="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50"
                    :disabled="!selectedItem || !editTitle.trim() || !editDue.trim() || isSavingEdit || isArchiving || isDeleting"
                  >
                    {{ isSavingEdit ? 'Saving...' : 'Save Changes' }}
                  </button>
                </div>
              </div>

              <p class="rounded-lg border border-amber-300 bg-amber-50 px-3 py-2 text-xs text-amber-800">
                Edit mode persists task details to backend and refreshes shared item state.
              </p>
            </form>
          </transition>
        </div>
      </aside>
    </div>
  </transition>
</template>
