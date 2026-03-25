<script setup lang="ts">
import { computed, ref, watch, onMounted, onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { Item } from '@/types/item';
import type { GraphAxisField, GraphTimeRangeKey, GraphVisualField, SelectOption } from '@/types/graph';

import { useAuth } from '@/composables/useAuth';
import { useItems } from '@/composables/useItems';
import { useSyncStatus } from '@/composables/useSyncStatus';
import { useSettings } from '@/composables/useSettings';

import ScatterPlot from '@/components/ScatterPlot.vue';
import TaskDrawer from '@/components/TaskDrawer.vue';
import AppHeader from '@/components/Header.vue';
import GraphControlBar from '@/components/GraphControl.vue';
import SpecialThanksModal from '@/components/SpecialThanks.vue';

const auth = useAuth();
const { items, isSyncing, fetchActiveItems, startNewSession, getCurrentToken, bindSyncStatusMap } = useItems();
const { syncMap, errorMap } = useSyncStatus();
const { displayUsername, isGuest } = auth;
const { t } = useSettings();

// --- Graph State ---
const selectedRange = ref<GraphTimeRangeKey>('1w');
const selectedYField = ref<GraphAxisField>('duration_minutes');
const selectedColorField = ref<GraphVisualField>('motivation');
const selectedRadiusField = ref<GraphVisualField>('duration_minutes');

const rangeOptions = computed<SelectOption<GraphTimeRangeKey>[]>(() => [
  { value: '1d', label: t('range1d') },
  { value: '3d', label: t('range3d') },
  { value: '1w', label: t('range1w') },
  { value: '2w', label: t('range2w') },
  { value: '1m', label: t('range1m') },
]);

const axisOptions = computed<SelectOption<GraphAxisField>[]>(() => [
  { value: 'duration_minutes', label: t('axisDuration') },
  { value: 'motivation', label: t('axisMotivation') },
  { value: 'status', label: t('axisStatus') },
]);

const visualOptions = computed<SelectOption<GraphVisualField>[]>(() => [
  { value: 'none', label: t('visualNone') },
  { value: 'duration_minutes', label: t('axisDuration') },
  { value: 'motivation', label: t('axisMotivation') },
  { value: 'status', label: t('axisStatus') },
]);

// --- UI State ---
type DrawerMode = 'create' | 'view' | 'edit' | 'tasks';

const isDrawerOpen = ref(false);
const isFullscreen = ref(false);
const isThanksOpen = ref(false);
const drawerMode = ref<DrawerMode>('view');
const selectedItem = ref<Item | null>(null);
const createSeed = ref<{ due: string; motivation: number } | null>(null);
const showWelcomeToast = ref(false);

let hasShownWelcomeToast = false;
let welcomeTimer: number | null = null;
let unlistenRemoteCatchup: UnlistenFn | null = null;
let hasCompletedInitialLoad = false;

// --- Logic ---
const hasResolvedUsername = computed(() => {
  const current = auth.username.value?.trim().toLowerCase();
  return Boolean(current && current !== 'unknown');
});

function triggerWelcomeToastOnce(): void {
  if (hasShownWelcomeToast || !hasResolvedUsername.value) return;
  hasShownWelcomeToast = true;
  showWelcomeToast.value = true;
  welcomeTimer = window.setTimeout(() => (showWelcomeToast.value = false), 2400);
}

async function loadItems(sessionToken: number): Promise<void> {
  await fetchActiveItems(sessionToken);
  if (sessionToken === getCurrentToken()) triggerWelcomeToastOnce();
}

const handleRefreshItems = () => loadItems(getCurrentToken());
bindSyncStatusMap(syncMap);

const handleLogout = async () => {
  if (confirm(t('logoutConfirm'))) {
    await auth.logout().catch(e => console.error(e));
  }
};

const openDrawer = (mode: DrawerMode) => {
  if (mode !== 'create') {
    createSeed.value = null;
  }
  drawerMode.value = mode;
  isDrawerOpen.value = true;
};

const handleSelectItem = (item: Item) => {
  createSeed.value = null;
  selectedItem.value = item;
  drawerMode.value = 'view';
  isDrawerOpen.value = true;
};

const handleRequestCreateFromPlot = (payload: { due: string; motivation: number }) => {
  createSeed.value = payload;
  drawerMode.value = 'create';
  isDrawerOpen.value = true;
};

const toggleFullscreen = () => {
  isFullscreen.value = !isFullscreen.value;
};

const handleGlobalKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Escape' && isFullscreen.value) {
    isFullscreen.value = false;
  }
};

// --- Lifecycles & Watchers ---
onMounted(async () => {
  if (!isGuest.value) {
    await auth.ensureUsername();
    unlistenRemoteCatchup = await listen('remote-catchup', () => handleRefreshItems());
  }
  await loadItems(startNewSession());
  hasCompletedInitialLoad = true;
  window.addEventListener('keydown', handleGlobalKeydown);
});

onUnmounted(() => {
  unlistenRemoteCatchup?.();
  if (welcomeTimer) clearTimeout(welcomeTimer);
  window.removeEventListener('keydown', handleGlobalKeydown);
});

watch(
  auth.username,
  async (nextUsername) => {
    if (!hasCompletedInitialLoad) return;
    if (isGuest.value) return;

    const normalized = nextUsername?.trim().toLowerCase();
    if (!normalized || normalized === 'unknown') {
      await auth.ensureUsername();
      return;
    }
    startNewSession();
    triggerWelcomeToastOnce();
    await handleRefreshItems(); 
  },
);
</script>

<template>
  <div class="app-container">
    <SpecialThanksModal 
      :show="isThanksOpen" 
      @close="isThanksOpen = false" 
    />

    <transition name="toast">
      <div v-if="showWelcomeToast" class="welcome-toast">
        {{ t('welcome') }} {{ displayUsername }}
      </div>
    </transition>

    <div class="main-layout" :class="{ 
      'drawer-open': isDrawerOpen,
      'is-fullscreen': isFullscreen 
    }">
      <template v-if="!isFullscreen">
        <AppHeader 
          :display-username="displayUsername" 
          :is-syncing="isSyncing" 
          :is-guest="isGuest"
          @logout="handleLogout"
          @show-thanks="isThanksOpen = true" 
          @show-help="isDrawerOpen = true" 
        />

        <GraphControlBar
          v-model:range="selectedRange"
          v-model:y-field="selectedYField"
          v-model:color-field="selectedColorField"
          v-model:radius-field="selectedRadiusField"
          :range-options="rangeOptions"
          :axis-options="axisOptions"
          :visual-options="visualOptions"
          @open-drawer="openDrawer"
        />
      </template>

      <main class="content-body" :class="{ 'fullscreen-mode': isFullscreen }">
        <section class="plot-container">
          <div class="plot-actions" role="toolbar" aria-label="Graph actions">
            <button
              class="plot-action-btn"
              type="button"
              :title="t('refresh')"
              @click="handleRefreshItems"
            >
              ↻
            </button>
            <button
              class="plot-action-btn plot-action-strong"
              type="button"
              :title="isFullscreen ? t('exitFullscreen') : t('maximizeGraph')"
              @click="toggleFullscreen"
            >
              {{ isFullscreen ? '☒' : '⛶' }}
            </button>
          </div>
          
          <ScatterPlot
            :items="items"
            v-model:range="selectedRange"
            v-model:y-field="selectedYField"
            v-model:color-field="selectedColorField"
            v-model:radius-field="selectedRadiusField"
            class="plot-component"
            @select-item="handleSelectItem"
            @request-create="handleRequestCreateFromPlot"
          />
        </section>
      </main>
    </div>

    <TaskDrawer
      :open="isDrawerOpen"
      :mode="drawerMode"
      :create-seed="createSeed"
      :selected-item="selectedItem"
      :items="items"
      :sync-map="syncMap"
      :error-map="errorMap"
      :is-syncing="isSyncing"
      @update:open="isDrawerOpen = $event"
      @update:mode="drawerMode = $event"
      @select-item="handleSelectItem"
      @success="handleRefreshItems" 
    />
  </div>
</template>

<style scoped>
/* --- Core Layout --- */
.app-container {
  position: relative;
  height: 100dvh;
  overflow: hidden;
  background: var(--tg-page-gradient);
}

.main-layout {
  display: flex;
  height: 100%;
  flex-direction: column;
  padding: 1rem;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

/* --- Fullscreen Logic --- */
.main-layout.is-fullscreen {
  padding: 0;
}

.content-body {
  margin-top: 0.75rem;
  display: flex;
  flex: 1;
  min-height: 0;
}

.content-body.fullscreen-mode {
  position: fixed;
  inset: 0;
  z-index: 40;
  margin: 0;
  padding: 1rem;
  background: var(--tg-surface-raised);
}

/* --- Plot Area (Added padding for axis spacing) --- */
.plot-container {
  flex: 1;
  position: relative;
  border-radius: 1rem;
  border: 1px solid var(--tg-border-default);
  background-color: var(--tg-surface);
  padding: 3.5rem 1.5rem 1rem 1rem;
  box-shadow: var(--tg-shadow-soft);
  display: flex;
  flex-direction: column;
}

.plot-actions {
  position: absolute;
  top: 0.85rem;
  right: 0.9rem;
  z-index: 10;
  display: inline-flex;
  gap: 0.5rem;
}

.plot-action-btn {
  min-width: 2.2rem;
  height: 2.2rem;
  border-radius: 0.6rem;
  border: 1px solid var(--tg-border-default);
  background: var(--tg-surface-translucent);
  color: var(--tg-text-default);
  font-size: 1rem;
  font-weight: 700;
  line-height: 1;
  cursor: pointer;
  backdrop-filter: blur(8px);
  box-shadow: var(--tg-shadow-soft);
  transition: transform 0.2s ease, background-color 0.2s ease, border-color 0.2s ease;
}

.plot-action-btn:hover {
  transform: translateY(-1px);
  background: var(--tg-surface);
  border-color: var(--tg-border-strong);
}

.plot-action-strong {
  font-size: 1.2rem;
}

.plot-component {
  flex: 1;
  width: 100%;
  height: 100%;
  min-height: 0;
}

/* --- Drawer Slide Adjustment --- */
@media (min-width: 1024px) {
  .main-layout.drawer-open:not(.is-fullscreen) {
    margin-right: 32rem;
  }
}

/* --- Welcome Toast --- */
.welcome-toast {
  position: fixed;
  left: 50%;
  top: 1.5rem;
  z-index: 30000;
  transform: translateX(-50%);
  border-radius: 0.75rem;
  border: 1px solid #6ee7b7;
  background-color: #ecfdf5;
  padding: 0.625rem 1.25rem;
  color: #065f46;
  font-weight: 600;
  box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
}

.toast-enter-active { transition: all 0.4s ease-out; }
.toast-leave-active { transition: all 0.3s ease-in; }
.toast-enter-from, .toast-leave-to { transform: translate(-50%, -1rem); opacity: 0; }
</style>