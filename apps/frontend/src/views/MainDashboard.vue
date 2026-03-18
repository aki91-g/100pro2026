<script setup lang="ts">
import { computed, ref, watch, onMounted, onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { Item } from '@/types/item';
import type { GraphAxisField, GraphTimeRangeKey, GraphVisualField } from '@/types/graph';

import { useAuth } from '@/composables/useAuth';
import { useItems } from '@/composables/useItems';
import { useSyncStatus } from '@/composables/useSyncStatus';

import ScatterPlot from '@/components/ScatterPlot.vue';
import TaskDrawer from '@/components/TaskDrawer.vue';

const auth = useAuth();


const {
  items,
  isSyncing,
  fetchActiveItems,
  startNewSession,
  getCurrentToken,
} = useItems();

const { syncMap, errorMap } = useSyncStatus();

const selectedRange = ref<GraphTimeRangeKey>('1w');
const selectedYField = ref<GraphAxisField>('duration_minutes');
const selectedColorField = ref<GraphVisualField>('motivation');
const selectedRadiusField = ref<GraphVisualField>('duration_minutes');

const rangeOptions: Array<{ value: GraphTimeRangeKey; label: string }> = [
  { value: '1d', label: '1 day' },
  { value: '3d', label: '3 days' },
  { value: '1w', label: '1 week' },
  { value: '2w', label: '2 weeks' },
  { value: '1m', label: '1 month' },
];

const axisOptions: Array<{ value: GraphAxisField; label: string }> = [
  { value: 'duration_minutes', label: 'Duration' },
  { value: 'motivation', label: 'Motivation' },
  { value: 'status', label: 'Status' },
];

const visualOptions: Array<{ value: GraphVisualField; label: string }> = [
  { value: 'none', label: 'None' },
  { value: 'duration_minutes', label: 'Duration' },
  { value: 'motivation', label: 'Motivation' },
  { value: 'status', label: 'Status' },
];

const isDrawerOpen = ref(false);
const drawerMode = ref<'create' | 'view' | 'edit' | 'tasks'>('view');
const selectedItem = ref<Item | null>(null);

const showWelcomeToast = ref(false);
let hasShownWelcomeToast = false;
let welcomeTimer: number | null = null;

type UUID = string;

interface RemoteCatchupEvent {
  user_id: UUID;
  synced_count: number;
}

let unlistenRemoteCatchup: UnlistenFn | null = null;

const hasResolvedUsername = computed(() => {
  const current = auth.username.value?.trim().toLowerCase();
  return Boolean(current && current !== 'unknown');
});

function triggerWelcomeToastOnce(): void {
  if (hasShownWelcomeToast || !hasResolvedUsername.value) return;

  hasShownWelcomeToast = true;
  showWelcomeToast.value = true;

  if (welcomeTimer !== null) {
    window.clearTimeout(welcomeTimer);
  }

  welcomeTimer = window.setTimeout(() => {
    showWelcomeToast.value = false;
    welcomeTimer = null;
  }, 2400);
}

async function loadItems(sessionToken: number): Promise<void> {
  await fetchActiveItems(sessionToken);
  if (sessionToken === getCurrentToken()) {
    triggerWelcomeToastOnce();
  }
}

async function handleRefreshItems(): Promise<void> {
  await loadItems(getCurrentToken());
}

async function handleLogout(): Promise<void> {
  if (!confirm('Are you sure you want to logout?')) return;
  try {
    await auth.logout();
  } catch (error) {
    console.error('Logout failed:', error);
  }
}

function openDrawer(mode: 'create' | 'view' | 'edit' | 'tasks'): void {
  drawerMode.value = mode;
  isDrawerOpen.value = true;
}

function handleSelectItem(item: Item): void {
  selectedItem.value = item;
  drawerMode.value = 'view';
  isDrawerOpen.value = true;
}

onMounted(async () => {
  await auth.ensureUsername();
  unlistenRemoteCatchup = await listen<RemoteCatchupEvent>('remote-catchup', () => {
    void handleRefreshItems();
  });

  const sessionToken = startNewSession();
  await loadItems(sessionToken);
});

onUnmounted(() => {
  if (unlistenRemoteCatchup) {
    unlistenRemoteCatchup();
    unlistenRemoteCatchup = null;
  }

  if (welcomeTimer !== null) {
    window.clearTimeout(welcomeTimer);
    welcomeTimer = null;
  }
});

watch(
  auth.username,
  async (nextUsername) => {
    const normalized = nextUsername?.trim().toLowerCase();
    if (!normalized || normalized === 'unknown') {
      await auth.ensureUsername();
      return;
    }

    triggerWelcomeToastOnce();
  },
  { immediate: true },
);

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
  { deep: true },
);
</script>

<template>
  <div class="relative h-dvh overflow-x-hidden bg-gradient-to-br from-slate-100 via-slate-50 to-blue-50">
    <transition
      enter-active-class="transition duration-300 ease-out"
      enter-from-class="translate-y-2 opacity-0"
      enter-to-class="translate-y-0 opacity-100"
      leave-active-class="transition duration-200 ease-in"
      leave-from-class="translate-y-0 opacity-100"
      leave-to-class="translate-y-2 opacity-0"
    >
      <div
        v-if="showWelcomeToast"
        class="fixed left-1/2 top-6 z-50 -translate-x-1/2 rounded-xl border border-emerald-300 bg-emerald-50 px-4 py-2 text-sm font-medium text-emerald-800 shadow-lg"
      >
        Welcome {{ auth.displayUsername }}
      </div>
    </transition>

    <div class="flex h-full min-h-0 flex-col p-4 transition-all duration-300 md:p-6" :class="isDrawerOpen ? 'lg:mr-[32rem]' : ''">
      <header class="rounded-2xl border border-slate-200/80 bg-white/90 px-4 py-3 shadow-sm backdrop-blur">
        <div class="flex flex-wrap items-center justify-between gap-3">
          <div class="flex items-center gap-3">
            <div>
              <h1 class="text-xl font-semibold tracking-tight text-slate-950 md:text-2xl">TaskGraph</h1>
            </div>
            <button type="button" class="rounded-full border border-slate-200 bg-white px-3 py-1 text-sm font-semibold shadow-sm">
              <span class="bg-gradient-to-r from-red-500 via-purple-500 to-blue-500 bg-clip-text text-transparent">100program v9</span>
            </button>
          </div>

          <div class="flex flex-wrap items-center justify-end gap-3">
            <span
              class="inline-flex items-center gap-2 rounded-full border border-slate-300 bg-slate-100 px-3 py-1 text-xs font-medium text-slate-700"
              :class="isSyncing ? 'border-blue-300 bg-blue-50 text-blue-700' : ''"
            >
              <span class="h-2 w-2 rounded-full" :class="isSyncing ? 'animate-pulse bg-blue-500' : 'bg-slate-400'" />
              {{ isSyncing ? 'Database syncing...' : 'Database idle' }}
            </span>

            <div class="inline-flex items-center gap-3 rounded-full border border-slate-200 bg-white px-3 py-1.5 text-sm text-slate-700 shadow-sm">
              <span class="font-medium text-slate-900">{{ auth.displayUsername }}</span>
              <button
                type="button"
                class="rounded-md border border-slate-300 bg-white px-2.5 py-1 text-xs font-medium text-slate-700 hover:bg-slate-100"
                @click="handleLogout"
              >
                Logout
              </button>
            </div>
          </div>
        </div>
      </header>

      <section class="mt-3 rounded-2xl border border-slate-200/80 bg-white/90 px-4 py-2.5 shadow-sm backdrop-blur">
        <div class="flex flex-wrap items-center justify-between gap-3">
          <div class="flex flex-1 flex-wrap items-center gap-3">
            <div class="flex min-w-[140px] flex-col">
              <label class="text-[10px] font-semibold uppercase tracking-[0.18em] text-slate-400">Window</label>
              <select v-model="selectedRange" class="rounded-lg border border-slate-300 bg-white px-3 py-1.5 text-sm text-slate-800 focus:border-blue-500 focus:outline-none">
              <option v-for="option in rangeOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
            </select>
          </div>

            <div class="flex min-w-[140px] flex-col">
              <label class="text-[10px] font-semibold uppercase tracking-[0.18em] text-slate-400">Y-Axis</label>
              <select v-model="selectedYField" class="rounded-lg border border-slate-300 bg-white px-3 py-1.5 text-sm text-slate-800 focus:border-blue-500 focus:outline-none">
              <option v-for="option in axisOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
            </select>
          </div>

            <div class="flex min-w-[140px] flex-col">
              <label class="text-[10px] font-semibold uppercase tracking-[0.18em] text-slate-400">Color</label>
              <select v-model="selectedColorField" class="rounded-lg border border-slate-300 bg-white px-3 py-1.5 text-sm text-slate-800 focus:border-blue-500 focus:outline-none">
              <option v-for="option in visualOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
            </select>
          </div>

            <div class="flex min-w-[140px] flex-col">
              <label class="text-[10px] font-semibold uppercase tracking-[0.18em] text-slate-400">Radius</label>
              <select v-model="selectedRadiusField" class="rounded-lg border border-slate-300 bg-white px-3 py-1.5 text-sm text-slate-800 focus:border-blue-500 focus:outline-none">
              <option v-for="option in visualOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
            </select>
            </div>
          </div>

          <div class="flex items-center gap-2">
            <button
              type="button"
              class="rounded-lg border border-slate-300 bg-white px-3 py-1.5 text-sm font-medium text-slate-700 hover:bg-slate-100"
              @click="handleRefreshItems"
            >
              Refresh
            </button>

            <button
              type="button"
              class="rounded-lg border border-slate-300 bg-white px-3 py-1.5 text-sm font-medium text-slate-700 hover:bg-slate-100"
              @click="openDrawer('tasks')"
            >
              Tasks
            </button>

            <button
              type="button"
              class="rounded-lg bg-blue-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-blue-700"
              @click="openDrawer('create')"
            >
              New Task
            </button>
          </div>
        </div>
      </section>

      <main class="mt-3 flex min-h-0 flex-1">
        <section class="flex min-h-0 flex-1 rounded-2xl border border-slate-200 bg-white p-2 shadow-sm md:p-3">
          <ScatterPlot
            :items="items"
            v-model:range="selectedRange"
            v-model:y-field="selectedYField"
            v-model:color-field="selectedColorField"
            v-model:radius-field="selectedRadiusField"
            class="min-h-0 flex-1"
            @select-item="handleSelectItem"
          />
        </section>
      </main>
    </div>

    <TaskDrawer
      :open="isDrawerOpen"
      :mode="drawerMode"
      :selected-item="selectedItem"
      :items="items"
      :sync-map="syncMap"
      :error-map="errorMap"
      :is-syncing="isSyncing"
      @update:open="isDrawerOpen = $event"
      @update:mode="drawerMode = $event"
      @select-item="handleSelectItem"
    />
  </div>
</template>
