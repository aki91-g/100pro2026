<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import type { Item } from '@/types/item';
import ActiveList from '@/components/ActiveList.vue';
import ArchivedList from '@/components/ArchivedList.vue';
import DeletedList from '@/components/DeletedList.vue';
import { useItems } from '@/composables/useItems';
import { useSettings } from '@/composables/useSettings';

type DrawerMode = 'create' | 'view' | 'edit' | 'tasks';
type TaskTab = 'active' | 'archived' | 'deleted';

const props = defineProps<{
  open: boolean;
  mode: DrawerMode;
  createSeed?: { due: string; motivation: number } | null;
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
  (event: 'success'): void;
}>();

const createTitle = ref('');
const createDescription = ref<string | null>(null);
const createDue = ref('');
const createDuration = ref<number | null>(null);
const createMotivation = ref(5);
const createStatusUi = ref<'backlog' | 'todo' | 'doing' | 'done'>('todo');

const editTitle = ref('');
const editDescription = ref<string | null>(null);
const editDue = ref('');
const editDuration = ref<number | null>(null);
const editMotivation = ref(5);
const selectedStatusUi = ref<'backlog' | 'todo' | 'doing' | 'done'>('todo');
const isSavingEdit = ref(false);
const isCreating = ref(false);
const isArchiving = ref(false);
const isDeleting = ref(false);
const isUpdatingStatus = ref(false);
const previousMode = ref<'view' | 'tasks'>('view');
const viewStatusSelectId = 'task-drawer-status-view';
const editStatusSelectId = 'task-drawer-status-edit';
const createStatusSelectId = 'task-drawer-status-create';
const currentTab = ref<TaskTab>('active');
const archivedItems = ref<Item[]>([]);
const deletedItems = ref<Item[]>([]);
const isLoadingArchived = ref(false);
const isLoadingDeleted = ref(false);
const errorArchived = ref<string | null>(null);
const errorDeleted = ref<string | null>(null);
const tabRequestToken = ref(0);
const isRestoringTabItem = ref(false);

const taskTabIds: Record<TaskTab, string> = {
  active: 'task-tab-active',
  archived: 'task-tab-archived',
  deleted: 'task-tab-deleted',
};

const taskTabPanelIds: Record<TaskTab, string> = {
  active: 'task-tabpanel-active',
  archived: 'task-tabpanel-archived',
  deleted: 'task-tabpanel-deleted',
};

const taskTabOrder: TaskTab[] = ['active', 'archived', 'deleted'];
const taskTabButtonRefs = ref<Record<TaskTab, HTMLButtonElement | null>>({
  active: null,
  archived: null,
  deleted: null,
});

const {
  createItem,
  updateItem,
  updateItemStatus,
  archiveItem,
  unarchiveItem,
  softDeleteItem,
  restoreItem,
  fetchArchivedItems,
  fetchDeletedItems,
  items: repositoryItems,
} = useItems();
const { t, language } = useSettings();

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

const isMutating = computed(() => isSavingEdit.value || isArchiving.value || isDeleting.value || isUpdatingStatus.value);
const drawerTitle = computed(() => (
  props.mode === 'create'
    ? t('drawerCreateTask')
    : props.mode === 'view'
      ? t('drawerDetails')
      : props.mode === 'edit'
        ? t('drawerEditTask')
        : t('list')
));

const localizedStatusOptions = computed(() => ([
  { value: 'backlog', label: t('statusBacklog') },
  { value: 'todo', label: t('statusTodo') },
  { value: 'doing', label: t('statusDoing') },
  { value: 'done', label: t('statusDone') },
]));

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
  createStatusUi.value = 'todo';

  const now = new Date();
  now.setMinutes(now.getMinutes() - now.getTimezoneOffset());
  createDue.value = now.toISOString().slice(0, 16);
}

function applyCreateSeed(): void {
  const seed = props.createSeed;
  if (!seed) return;

  const dueDate = new Date(seed.due);
  if (!Number.isNaN(dueDate.getTime())) {
    createDue.value = toDatetimeLocal(dueDate.toISOString());
  }

  createMotivation.value = Math.min(10, Math.max(1, Math.round(seed.motivation)));
}

function hydrateEditForm(item: Item | null): void {
  if (!item) {
    editTitle.value = '';
    editDescription.value = null;
    editDue.value = '';
    editDuration.value = null;
    editMotivation.value = 5;
    selectedStatusUi.value = 'todo';
    return;
  }

  editTitle.value = item.title;
  editDescription.value = item.description;
  editDue.value = toDatetimeLocal(item.due);
  editDuration.value = item.duration_minutes;
  editMotivation.value = typeof item.motivation === 'number' ? item.motivation : 5;
  selectedStatusUi.value = toUiStatus(item.status);
}

function formatDateForLocale(isoValue: string): string {
  const date = new Date(isoValue);
  if (Number.isNaN(date.getTime())) return '';
  const locale = language.value === 'ja' ? 'ja-JP' : 'en-US';
  return date.toLocaleString(locale, {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  });
}

function toUiStatus(status: Item['status']): 'backlog' | 'todo' | 'doing' | 'done' {
  if (status === 'backlog') return 'backlog';
  if (status === 'done') return 'done';
  if (status === 'inprogress') return 'doing';
  return 'todo';
}

function fromUiStatus(status: 'backlog' | 'todo' | 'doing' | 'done'): Item['status'] {
  if (status === 'backlog') return 'backlog';
  if (status === 'doing') return 'inprogress';
  if (status === 'done') return 'done';
  return 'todo';
}

function goToTasks(): void {
  emit('update:mode', 'tasks');
}

function setCurrentTab(tab: TaskTab): void {
  currentTab.value = tab;
}

function setTaskTabButtonRef(tab: TaskTab, element: Element | null): void {
  taskTabButtonRefs.value[tab] = element as HTMLButtonElement | null;
}

function focusTaskTab(tab: TaskTab): void {
  taskTabButtonRefs.value[tab]?.focus();
}

function moveTabFocus(current: TaskTab, direction: 1 | -1): void {
  const currentIndex = taskTabOrder.indexOf(current);
  const nextIndex = (currentIndex + direction + taskTabOrder.length) % taskTabOrder.length;
  const nextTab = taskTabOrder[nextIndex] ?? current;
  focusTaskTab(nextTab);
}

function handleTaskTabKeydown(event: KeyboardEvent, tab: TaskTab): void {
  if (event.key === 'ArrowRight') {
    event.preventDefault();
    moveTabFocus(tab, 1);
    return;
  }

  if (event.key === 'ArrowLeft') {
    event.preventDefault();
    moveTabFocus(tab, -1);
    return;
  }

  if (event.key === 'Home') {
    event.preventDefault();
    focusTaskTab('active');
    return;
  }

  if (event.key === 'End') {
    event.preventDefault();
    focusTaskTab('deleted');
    return;
  }

  if (event.key === 'Enter' || event.key === ' ') {
    event.preventDefault();
    setCurrentTab(tab);
  }
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

async function loadTaskTabItems(): Promise<void> {
  if (props.mode !== 'tasks' || !props.open) return;

  if (currentTab.value === 'active') return;

  const tab = currentTab.value;
  const token = ++tabRequestToken.value;

  if (tab === 'archived') {
    isLoadingArchived.value = true;
    errorArchived.value = null;
    try {
      const result = await fetchArchivedItems();
      if (token !== tabRequestToken.value || currentTab.value !== tab) return;
      archivedItems.value = result;
    } catch (error) {
      if (token !== tabRequestToken.value || currentTab.value !== tab) return;
      errorArchived.value = 'Failed to load archived items.';
      console.error('Failed to load archived tab items:', error);
    } finally {
      if (token === tabRequestToken.value && currentTab.value === tab) {
        isLoadingArchived.value = false;
      }
    }
    return;
  }

  isLoadingDeleted.value = true;
  errorDeleted.value = null;
  try {
    const result = await fetchDeletedItems();
    if (token !== tabRequestToken.value || currentTab.value !== tab) return;
    deletedItems.value = result;
  } catch (error) {
    if (token !== tabRequestToken.value || currentTab.value !== tab) return;
    errorDeleted.value = 'Failed to load deleted items.';
    console.error('Failed to load deleted tab items:', error);
  } finally {
    if (token === tabRequestToken.value && currentTab.value === tab) {
      isLoadingDeleted.value = false;
    }
  }
}

async function handleUnarchiveFromList(item: Item): Promise<void> {
  if (isRestoringTabItem.value) return;

  isRestoringTabItem.value = true;
  try {
    await unarchiveItem(item.id);
    archivedItems.value = archivedItems.value.filter((entry) => entry.id !== item.id);
    emit('success');
  } catch (error) {
    console.error('Failed to unarchive item from list:', error);
  } finally {
    isRestoringTabItem.value = false;
  }
}

async function handleRestoreFromList(item: Item): Promise<void> {
  if (isRestoringTabItem.value) return;

  isRestoringTabItem.value = true;
  try {
    await restoreItem(item.id);
    deletedItems.value = deletedItems.value.filter((entry) => entry.id !== item.id);
    emit('success');
  } catch (error) {
    console.error('Failed to restore item from list:', error);
  } finally {
    isRestoringTabItem.value = false;
  }
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

    const createStatus = fromUiStatus(createStatusUi.value);
    if (createStatus !== 'todo') {
      await updateItemStatus(id, createStatus);
    }

    const created = repositoryItems.value.find((item) => item.id === id) ?? null;
    if (created) {
      emit('select-item', created);
      emit('update:mode', 'view');
      emit('success');
    }
  } catch (error) {
    console.error('Failed to create item:', error);
  } finally {
    isCreating.value = false;
  }
}

async function handleArchive(): Promise<void> {
  if (isMutating.value || !props.selectedItem) return;

  isArchiving.value = true;
  try {
    await archiveItem(props.selectedItem.id);
    emit('success');
    emit('update:open', false);
  } catch (error) {
    console.error('Failed to archive item:', error);
  } finally {
    isArchiving.value = false;
  }
}

async function handleDelete(): Promise<void> {
  if (isMutating.value || !props.selectedItem) return;
  if (!confirm(t('drawerDeleteConfirm'))) return;

  isDeleting.value = true;
  try {
    await softDeleteItem(props.selectedItem.id);
    emit('success');
    emit('update:open', false);
  } catch (error) {
    console.error('Failed to delete item:', error);
  } finally {
    isDeleting.value = false;
  }
}

async function handleEditSubmit(): Promise<void> {
  if (isMutating.value) return;
  if (!props.selectedItem || !editTitle.value.trim() || !editDue.value.trim()) return;

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

    emit('success');
    
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

async function handleStatusChange(): Promise<void> {
  if (!props.selectedItem || isMutating.value) return;

  const selectedItemSnapshot = props.selectedItem;
  const selectedItemId = props.selectedItem.id;
  const originalStatus = props.selectedItem.status;
  const originalSyncStatus = props.selectedItem.sync_status;

  const nextStatus = fromUiStatus(selectedStatusUi.value);
  if (originalStatus === nextStatus) return;

  isUpdatingStatus.value = true;
  try {
    await updateItemStatus(selectedItemId, nextStatus);
    emit('select-item', {
      ...selectedItemSnapshot,
      id: selectedItemId,
      status: nextStatus,
      sync_status: originalSyncStatus === 'local_only' ? 'local_only' : 'modified',
      updated_at: new Date().toISOString(),
    });
  } catch (error) {
    console.error('Failed to update item status:', error);
    selectedStatusUi.value = toUiStatus(originalStatus);
  } finally {
    isUpdatingStatus.value = false;
  }
}

watch(
  () => props.mode,
  (mode) => {
    if (mode === 'create') {
      resetCreateForm();
      applyCreateSeed();
    }
    if (mode === 'tasks') {
      currentTab.value = 'active';
      void loadTaskTabItems();
    }
  },
  { immediate: true },
);

watch(
  () => props.createSeed,
  () => {
    if (props.mode === 'create' && props.open) {
      applyCreateSeed();
    }
  },
  { deep: true },
);

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen && props.mode === 'create') {
      resetCreateForm();
      applyCreateSeed();
    }
    if (isOpen && props.mode === 'tasks') {
      currentTab.value = 'active';
      void loadTaskTabItems();
    }
  },
);

watch(currentTab, () => {
  void loadTaskTabItems();
});

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
  <transition name="drawer-slide">
    <div v-if="open" class="drawer-overlay">
      <div class="drawer-backdrop" @click="closeDrawer" />

      <aside role="dialog" aria-modal="true" aria-labelledby="task-drawer-title" class="drawer-content">
        <div class="bg-glow">
          <div class="glow-orb orb-red"></div>
          <div class="glow-orb orb-blue"></div>
          <div class="glow-center"></div>
        </div>

        <header class="drawer-header">
          <div class="header-left">
            <button v-if="mode === 'view'" type="button" class="icon-button-outline" @click="goToTasks">
              <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
              </svg>
              <span>{{ t('drawerBack') }}</span>
            </button>
            <h2 id="task-drawer-title" class="drawer-title" :class="{ 'is-hidden-title': mode === 'tasks' }">{{ drawerTitle }}</h2>
          </div>

          <div class="header-right">
            <button v-if="mode === 'tasks'" type="button" class="primary-button-sm" @click="goToCreate">
              <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
              </svg>
              <span>{{ t('new') }}</span>
            </button>
            <button type="button" class="secondary-button-sm" @click="closeDrawer">{{ t('drawerClose') }}</button>
          </div>
        </header>

        <div class="drawer-body">
          <nav v-if="mode === 'tasks'" class="tab-container" role="tablist" :aria-label="t('list')">
            <button
              :id="taskTabIds.active"
              :ref="(el) => setTaskTabButtonRef('active', el as Element | null)"
              type="button"
              class="tab-trapezoid"
              :class="{ 'active': currentTab === 'active' }"
              role="tab"
              :aria-selected="currentTab === 'active'"
              :aria-controls="taskTabPanelIds.active"
              :tabindex="currentTab === 'active' ? 0 : -1"
              @keydown="handleTaskTabKeydown($event, 'active')"
              @click="setCurrentTab('active')"
            >
              {{ t('active') }}
            </button>
            <button
              :id="taskTabIds.archived"
              :ref="(el) => setTaskTabButtonRef('archived', el as Element | null)"
              type="button"
              class="tab-trapezoid"
              :class="{ 'active': currentTab === 'archived' }"
              role="tab"
              :aria-selected="currentTab === 'archived'"
              :aria-controls="taskTabPanelIds.archived"
              :tabindex="currentTab === 'archived' ? 0 : -1"
              @keydown="handleTaskTabKeydown($event, 'archived')"
              @click="setCurrentTab('archived')"
            >
              {{ t('archived') }}
            </button>
            <button
              :id="taskTabIds.deleted"
              :ref="(el) => setTaskTabButtonRef('deleted', el as Element | null)"
              type="button"
              class="tab-trapezoid"
              :class="{ 'active': currentTab === 'deleted' }"
              role="tab"
              :aria-selected="currentTab === 'deleted'"
              :aria-controls="taskTabPanelIds.deleted"
              :tabindex="currentTab === 'deleted' ? 0 : -1"
              @keydown="handleTaskTabKeydown($event, 'deleted')"
              @click="setCurrentTab('deleted')"
            >
              {{ t('deleted') }}
            </button>
          </nav>

          <transition name="fade-slide" mode="out-in">
            <form v-if="mode === 'create'" key="create" @submit.prevent="submitCreate" class="task-form">
              <div class="edit-fields">
                <div class="input-group">
                  <label>{{ t('drawerTitleRequired') }}</label>
                  <input v-model="createTitle" type="text" :placeholder="t('drawerTitlePlaceholder')" :disabled="isCreating" class="user-input" />
                </div>

                <div class="input-group">
                  <label>{{ t('drawerDescription') }}</label>
                  <textarea v-model="createDescription" rows="6" :placeholder="t('drawerDescriptionPlaceholder')" :disabled="isCreating" class="user-input" />
                </div>

                <div class="input-group">
                  <label>{{ t('drawerDueRequired') }}</label>
                  <input v-model="createDue" type="datetime-local" :disabled="isCreating" class="user-input" />
                </div>

                <div class="input-group">
                  <label>{{ t('drawerDurationMin') }}</label>
                  <input v-model.number="createDuration" type="number" min="1" :disabled="isCreating" class="user-input" />
                </div>

                <div class="input-group">
                  <label>{{ t('drawerMotivation') }}</label>
                  <div class="motivation-field">
                    <input
                      v-model.number="createMotivation"
                      type="range"
                      min="1"
                      max="10"
                      step="1"
                      :disabled="isCreating"
                      class="motivation-slider"
                    />
                    <span class="motivation-value">{{ createMotivation }}</span>
                  </div>
                </div>

                <div class="input-group">
                  <label :for="createStatusSelectId">{{ t('drawerStatus') }}</label>
                  <select
                    :id="createStatusSelectId"
                    v-model="createStatusUi"
                    :disabled="isCreating"
                    class="user-input status-select"
                    :aria-label="t('drawerStatusAriaEdit')"
                  >
                    <option v-for="statusOption in localizedStatusOptions" :key="statusOption.value" :value="statusOption.value">{{ statusOption.label }}</option>
                  </select>
                </div>
              </div>

              <div class="edit-footer">
                <button type="button" class="secondary-action" @click="cancelCreate">{{ t('drawerCancel') }}</button>
                <button type="submit" :disabled="!createTitle.trim() || !createDue.trim() || isCreating" class="primary-button">
                  {{ isCreating ? t('drawerCreating') : t('drawerCreateTask') }}
                </button>
              </div>
            </form>

            <div v-else-if="mode === 'view'" key="view" class="view-content">
              <div v-if="selectedItem" class="detail-card">
                <div class="detail-header">
                  <h3>{{ selectedItem.title }}</h3>
                  <button type="button" class="edit-link" @click="startEdit">{{ t('drawerEdit') }}</button>
                </div>
                <p class="detail-desc">{{ selectedItem.description || t('drawerNoDescription') }}</p>
                
                <div class="detail-grid">
                  <div class="grid-item">
                    <label class="label" :for="viewStatusSelectId">{{ t('drawerStatus') }}</label>
                    <select
                      :id="viewStatusSelectId"
                      v-model="selectedStatusUi"
                      class="status-select"
                      :disabled="isMutating"
                      :aria-label="t('drawerStatusAriaView')"
                      @change="handleStatusChange"
                    >
                      <option v-for="statusOption in localizedStatusOptions" :key="statusOption.value" :value="statusOption.value">{{ statusOption.label }}</option>
                    </select>
                  </div>
                  <div class="grid-item"><span class="label">{{ t('drawerDue') }}</span><p>{{ formatDateForLocale(selectedItem.due) }}</p></div>
                  <div class="grid-item"><span class="label">{{ t('drawerMotivation') }}</span><p>{{ selectedItem.motivation ?? '5' }}</p></div>
                  <div class="grid-item"><span class="label">{{ t('drawerDuration') }}</span><p>{{ selectedItem.duration_minutes ?? '---' }} {{ t('drawerMinuteUnit') }}</p></div>
                </div>

                <div class="danger-zone">
                  <button type="button" class="danger-button-outline" @click="handleArchive" :disabled="isMutating">{{ t('drawerArchive') }}</button>
                  <button type="button" class="danger-button-outline" @click="handleDelete" :disabled="isMutating">{{ t('drawerDelete') }}</button>
                </div>
              </div>
            </div>

            <div v-else-if="mode === 'tasks'" key="tasks" class="task-list-wrapper">
              <section
                v-show="currentTab === 'active'"
                :id="taskTabPanelIds.active"
                role="tabpanel"
                :aria-labelledby="taskTabIds.active"
                tabindex="0"
              >
                <ActiveList
                  :items="items"
                  :sync-map="strictSyncMap"
                  :error-map="strictErrorMap"
                  :is-syncing="isSyncing"
                  @select-item="handleTaskListSelect"
                />
                <div v-if="items.length === 0" class="tab-state-message">{{ t('drawerNoTasksInTab') }}</div>
              </section>

              <section
                v-show="currentTab === 'archived'"
                :id="taskTabPanelIds.archived"
                role="tabpanel"
                :aria-labelledby="taskTabIds.archived"
                tabindex="0"
              >
                <ArchivedList
                  :items="archivedItems"
                  :sync-map="strictSyncMap"
                  :error-map="strictErrorMap"
                  :is-syncing="isSyncing"
                  :is-processing="isRestoringTabItem"
                  @unarchive-item="handleUnarchiveFromList"
                />
                <div v-if="isLoadingArchived" class="tab-state-message">{{ t('dbSyncing') }}</div>
                <div v-else-if="errorArchived" class="tab-state-message tab-state-error">{{ errorArchived }}</div>
                <div v-else-if="archivedItems.length === 0" class="tab-state-message">{{ t('drawerNoTasksInTab') }}</div>
              </section>

              <section
                v-show="currentTab === 'deleted'"
                :id="taskTabPanelIds.deleted"
                role="tabpanel"
                :aria-labelledby="taskTabIds.deleted"
                tabindex="0"
              >
                <DeletedList
                  :items="deletedItems"
                  :sync-map="strictSyncMap"
                  :error-map="strictErrorMap"
                  :is-syncing="isSyncing"
                  :is-processing="isRestoringTabItem"
                  @restore-item="handleRestoreFromList"
                />
                <div v-if="isLoadingDeleted" class="tab-state-message">{{ t('dbSyncing') }}</div>
                <div v-else-if="errorDeleted" class="tab-state-message tab-state-error">{{ errorDeleted }}</div>
                <div v-else-if="deletedItems.length === 0" class="tab-state-message">{{ t('drawerNoTasksInTab') }}</div>
              </section>
            </div>

            <form v-else key="edit" class="task-form edit-form" @submit.prevent="handleEditSubmit">
              <div class="edit-fields">
                <div class="input-group">
                  <label>{{ t('drawerTitle') }}</label>
                  <input v-model="editTitle" type="text" :disabled="isSavingEdit" class="user-input" />
                </div>
                <div class="input-group">
                  <label>{{ t('drawerDescription') }}</label>
                  <textarea v-model="editDescription" rows="6" :disabled="isSavingEdit" class="user-input" />
                </div>

                <div class="input-group">
                  <label>{{ t('drawerDue') }}</label>
                  <input v-model="editDue" type="datetime-local" :disabled="isSavingEdit" class="user-input" />
                </div>

                <div class="input-group">
                  <label>{{ t('drawerDurationMin') }}</label>
                  <input v-model.number="editDuration" type="number" min="1" :disabled="isSavingEdit" class="user-input" />
                </div>

                <div class="input-group">
                  <label>{{ t('drawerMotivation') }}</label>
                  <div class="motivation-field">
                    <input
                      v-model.number="editMotivation"
                      type="range"
                      min="1"
                      max="10"
                      step="1"
                      :disabled="isSavingEdit"
                      class="motivation-slider"
                    />
                    <span class="motivation-value">{{ editMotivation }}</span>
                  </div>
                </div>

                <div class="input-group">
                  <label :for="editStatusSelectId">{{ t('drawerStatus') }}</label>
                  <select
                    :id="editStatusSelectId"
                    v-model="selectedStatusUi"
                    :disabled="isMutating"
                    class="user-input status-select"
                    :aria-label="t('drawerStatusAriaEdit')"
                    @change="handleStatusChange"
                  >
                    <option v-for="statusOption in localizedStatusOptions" :key="statusOption.value" :value="statusOption.value">{{ statusOption.label }}</option>
                  </select>
                </div>

                <div class="danger-zone">
                  <button type="button" class="danger-button-outline" @click="handleArchive" :disabled="isMutating">{{ t('drawerArchive') }}</button>
                  <button type="button" class="danger-button-outline" @click="handleDelete" :disabled="isMutating">{{ t('drawerDelete') }}</button>
                </div>
              </div>

              <div class="edit-footer">
                <button type="button" class="secondary-action" @click="cancelEdit">{{ t('drawerCancel') }}</button>
                <button type="submit" :disabled="!editTitle.trim() || !editDue.trim() || isMutating" class="primary-button">
                  {{ t('drawerSaveChanges') }}
                </button>
              </div>
            </form>
          </transition>
        </div>
      </aside>
    </div>
  </transition>
</template>

<style scoped>
:global(.dark) .drawer-content {
  --bg-primary: var(--tg-surface);
  --bg-secondary: var(--tg-surface-raised);
}

/* --- Layout & Overlay --- */
.drawer-overlay {
  position: fixed;
  inset: 0;
  z-index: 50;
  display: flex;
  justify-content: flex-end;
}

.drawer-backdrop {
  position: absolute;
  inset: 0;
  background: rgba(15, 23, 42, 0.1);
  backdrop-filter: blur(4px);
}

.drawer-content {
  position: relative;
  width: 100%;
  max-width: 500px;
  height: 100%;
  background: color-mix(in srgb, var(--bg-primary) 85%, transparent);
  backdrop-filter: blur(30px);
  border-left: 1px solid color-mix(in srgb, var(--bg-primary) 55%, transparent);
  box-shadow: -10px 0 50px rgba(0, 0, 0, 0.05);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* --- Background Orbs (Login Screen Theme) --- */
.bg-glow {
  position: absolute;
  inset: 0;
  z-index: -1;
  pointer-events: none;
}

.glow-orb {
  position: absolute;
  width: 400px;
  height: 400px;
  border-radius: 50%;
  filter: blur(80px);
  opacity: 0.12;
}

.orb-red { top: -10%; right: -10%; background: radial-gradient(circle, #ef4444, transparent); }
.orb-blue { bottom: -10%; left: -10%; background: radial-gradient(circle, #3b82f6, transparent); }
.glow-center {
  position: absolute;
  top: 40%; left: 30%;
  width: 300px; height: 300px;
  background: radial-gradient(circle, rgba(168, 85, 247, 0.2), transparent);
  filter: blur(60px);
}

/* --- Header --- */
.drawer-header {
  padding: 1.5rem 2rem;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid var(--tg-border-default);
}

.header-left, .header-right {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.drawer-title {
  font-size: 1.25rem;
  font-weight: 800;
  color: var(--text-strong);
  letter-spacing: -0.02em;
}

.is-hidden-title {
  position: absolute;
  width: 1px;
  height: 1px;
  overflow: hidden;
  clip: rect(0 0 0 0);
  clip-path: inset(50%);
  white-space: nowrap;
}

.tab-container {
  display: flex;
  align-items: center;
  gap: 0.45rem;
  justify-content: flex-start;
  margin-bottom: 1rem;
}

.tab-trapezoid {
  border: 1px solid var(--tg-border-default);
  background: color-mix(in srgb, var(--bg-primary) 80%, transparent);
  color: var(--text-primary);
  font-size: 0.78rem;
  font-weight: 700;
  letter-spacing: 0.01em;
  padding: 0.44rem 0.85rem;
  cursor: pointer;
  clip-path: polygon(10% 0, 100% 0, 90% 100%, 0 100%);
  transition: transform 0.2s ease, border-color 0.2s ease, filter 0.2s ease;
}

.tab-trapezoid:hover {
  border-color: #a855f7;
  transform: translateY(-1px);
}

.tab-trapezoid.active {
  border-color: transparent;
  color: #ffffff;
  background: linear-gradient(135deg, #ef4444 0%, #a855f7 50%, #3b82f6 100%);
  box-shadow: 0 5px 16px rgba(168, 85, 247, 0.24);
}

/* --- Body & Forms --- */
.drawer-body {
  flex: 1;
  overflow-y: auto;
  padding: 2rem;
  min-height: 0;
}

.task-form {
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.task-list-wrapper {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.tab-state-message {
  border: 1px dashed var(--tg-border-default);
  border-radius: 0.85rem;
  padding: 1rem;
  text-align: center;
  color: var(--text-muted);
  font-size: 0.86rem;
  font-weight: 600;
}

.tab-state-error {
  border-color: #fecaca;
  color: #b91c1c;
  background: #fef2f2;
}

.edit-form {
  height: 100%;
  min-height: 0;
  gap: 0;
}

.edit-fields {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding-right: 0.25rem;
  padding-bottom: 1.25rem;
}

.input-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
}

.input-row-3 {
  grid-template-columns: repeat(3, 1fr);
}

.input-group label {
  display: block;
  font-size: 0.75rem;
  font-weight: 700;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 0.4rem;
}

.user-input {
  width: 100%;
  background: var(--bg-primary);
  border: 1px solid var(--tg-border-default);
  border-radius: 0.75rem;
  padding: 0.7rem 0.9rem;
  color: var(--text-primary);
  font-size: 0.95rem;
  transition: border-color 0.24s cubic-bezier(0.16, 1, 0.3, 1), box-shadow 0.24s cubic-bezier(0.16, 1, 0.3, 1), background 0.24s cubic-bezier(0.16, 1, 0.3, 1);
}

.user-input:focus {
  outline: none;
  border-color: #64748b;
  background: var(--bg-primary);
  box-shadow: 0 0 0 3px rgba(100, 116, 139, 0.15);
}

/* --- Buttons --- */
.primary-button {
  background: linear-gradient(135deg, #ef4444 0%, #a855f7 50%, #3b82f6 100%);
  color: white;
  padding: 0.7rem 1.2rem;
  border-radius: 0.75rem;
  font-weight: 700;
  border: none;
  cursor: pointer;
  transition: transform 0.24s cubic-bezier(0.16, 1, 0.3, 1), box-shadow 0.24s cubic-bezier(0.16, 1, 0.3, 1), filter 0.24s cubic-bezier(0.16, 1, 0.3, 1);
  box-shadow: 0 4px 15px rgba(168, 85, 247, 0.25);
}

.primary-button:disabled {
  background: var(--bg-secondary);
  color: var(--text-muted);
  box-shadow: none;
  cursor: not-allowed;
}

.primary-button-sm {
  background: linear-gradient(135deg, #ef4444 0%, #a855f7 50%, #3b82f6 100%);
  color: white;
  padding: 0.5rem 1rem;
  border-radius: 8px;
  font-size: 0.875rem;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 0.4rem;
}

.secondary-button-sm {
  background: var(--bg-primary);
  border: 1px solid var(--tg-border-default);
  color: var(--text-primary);
  padding: 0.5rem 1rem;
  border-radius: 8px;
  font-size: 0.875rem;
  cursor: pointer;
}

.icon-button-outline {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  color: var(--text-primary);
  font-size: 0.875rem;
  font-weight: 600;
}

.link-text {
  color: var(--text-muted);
  font-weight: 600;
  cursor: pointer;
}

.form-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 1.5rem;
  margin-top: 1rem;
}

.edit-footer {
  position: sticky;
  bottom: 0;
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: 0.75rem;
  padding: 1.5rem 0 0;
  background: transparent; 
  box-shadow: none; 
  border-top: none; 
}

.secondary-action {
  border: 1px solid var(--tg-border-default);
  background: var(--bg-secondary);
  color: var(--text-primary);
  border-radius: 0.75rem;
  padding: 0.7rem 1rem;
  font-weight: 600;
  cursor: pointer;
  transition: border-color 0.2s ease, background-color 0.2s ease, color 0.2s ease;
}

.secondary-action:hover {
  border-color: var(--tg-border-strong);
  background: color-mix(in srgb, var(--bg-secondary) 75%, var(--bg-primary));
  color: var(--text-strong);
}

/* --- Details View --- */
.detail-card {
  background: color-mix(in srgb, var(--bg-primary) 72%, transparent);
  border: 1px solid color-mix(in srgb, var(--tg-border-default) 82%, transparent);
  border-radius: 20px;
  padding: 1.5rem;
}

.detail-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 1rem;
}

.detail-header h3 {
  font-size: 1.25rem;
  font-weight: 800;
  color: var(--text-strong);
}

.edit-link {
  color: #a855f7;
  font-weight: 700;
  font-size: 0.875rem;
}

.detail-desc {
  color: var(--text-primary);
  font-size: 0.95rem;
  margin-bottom: 1.5rem;
  line-height: 1.6;
}

.detail-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1.25rem;
}

.grid-item .label {
  display: block;
  font-size: 0.65rem;
  font-weight: 800;
  color: var(--text-muted);
  text-transform: uppercase;
  margin-bottom: 0.25rem;
}

.grid-item p {
  color: var(--text-primary);
  font-weight: 600;
  font-size: 0.9rem;
}

.status-select {
  width: 100%;
  background: var(--bg-primary);
  border: 1px solid var(--tg-border-default);
  border-radius: 0.75rem;
  padding: 0.7rem 0.9rem;
  color: var(--text-primary);
  font-size: 0.92rem;
  font-weight: 600;
  text-transform: lowercase;
  cursor: pointer;
}

.status-select:hover {
  border-color: var(--tg-border-strong);
  box-shadow: 0 2px 10px rgba(15, 23, 42, 0.06);
}

.status-select:focus {
  outline: none;
  border-color: var(--tg-border-strong);
  box-shadow: 0 0 0 3px rgba(100, 116, 139, 0.18);
}

.motivation-field {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.motivation-slider {
  flex: 1;
  accent-color: #64748b;
  cursor: pointer;
}

.motivation-value {
  min-width: 2rem;
  text-align: right;
  font-size: 0.88rem;
  font-weight: 700;
  color: var(--text-primary);
}

/* --- Danger Zone --- */
.danger-zone {
  display: flex;
  gap: 0.75rem;
  margin: 1rem;
  padding: 1rem;
  border-radius: 0.75rem;
  border: 1px solid #fecaca;
  background: #fef2f2;
}

.danger-button-outline {
  flex: 1;
  padding: 0.6rem;
  border: 1px solid #fecaca;
  background: var(--bg-primary);
  color: #dc2626;
  border-radius: 0.75rem;
  font-size: 0.8rem;
  font-weight: 600;
  cursor: pointer;
  transition: border-color 0.2s ease, background-color 0.2s ease;
}

.danger-button-outline:hover {
  border-color: #f87171;
  background: #fff1f2;
}

/* --- Transitions --- */
.drawer-slide-enter-active, .drawer-slide-leave-active { transition: transform 0.38s cubic-bezier(0.22, 1, 0.36, 1); }
.drawer-slide-enter-from, .drawer-slide-leave-to { transform: translateX(100%); }

.fade-slide-enter-active, .fade-slide-leave-active { transition: all 0.28s cubic-bezier(0.22, 1, 0.36, 1); }
.fade-slide-enter-from { opacity: 0; transform: translateY(10px); }
.fade-slide-leave-to { opacity: 0; transform: translateY(-10px); }

@media (max-width: 640px) {
  .drawer-header {
    padding: 1rem 1rem;
  }

  .drawer-body {
    padding: 1rem;
  }
}
</style>