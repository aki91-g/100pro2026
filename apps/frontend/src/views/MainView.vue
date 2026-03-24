<script setup lang="ts">
import { computed, ref, watch, onMounted, onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { Item } from '@/types/item';
import type { GraphAxisField, GraphTimeRangeKey, GraphVisualField, SelectOption } from '@/types/graph';

import { useAuth } from '@/composables/useAuth';
import { useItems } from '@/composables/useItems';
import { useSyncStatus } from '@/composables/useSyncStatus';

import ScatterPlot from '@/components/ScatterPlot.vue';
import TaskDrawer from '@/components/TaskDrawer.vue';
import AppHeader from '@/components/Header.vue';
import GraphControlBar from '@/components/GraphControl.vue';
import SpecialThanksModal from '@/components/SpecialThanks.vue';

const auth = useAuth();
const { items, isSyncing, fetchActiveItems, startNewSession, getCurrentToken, bindSyncStatusMap } = useItems();
const { syncMap, errorMap } = useSyncStatus();
const { displayUsername } = auth;

// --- Graph State ---
const selectedRange = ref<GraphTimeRangeKey>('1w');
const selectedYField = ref<GraphAxisField>('duration_minutes');
const selectedColorField = ref<GraphVisualField>('motivation');
const selectedRadiusField = ref<GraphVisualField>('duration_minutes');

const rangeOptions: SelectOption<GraphTimeRangeKey>[] = [
  { value: '1d', label: '1 day' }, { value: '3d', label: '3 days' },
  { value: '1w', label: '1 week' }, { value: '2w', label: '2 weeks' },
  { value: '1m', label: '1 month' },
];

const axisOptions: SelectOption<GraphAxisField>[] = [
  { value: 'duration_minutes', label: 'Duration' },
  { value: 'motivation', label: 'Motivation' },
  { value: 'status', label: 'Status' },
];

const visualOptions: SelectOption<GraphVisualField>[] = [
  { value: 'none', label: 'None' },
  { value: 'duration_minutes', label: 'Duration' },
  { value: 'motivation', label: 'Motivation' },
  { value: 'status', label: 'Status' },
];

// --- UI State ---
type DrawerMode = 'create' | 'view' | 'edit' | 'tasks';

const isDrawerOpen = ref(false);
const isFullscreen = ref(false);
const isThanksOpen = ref(false);
const drawerMode = ref<DrawerMode>('view');
const selectedItem = ref<Item | null>(null);
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
  if (confirm('Are you sure you want to logout?')) {
    await auth.logout().catch(e => console.error(e));
  }
};

const openDrawer = (mode: DrawerMode) => {
  drawerMode.value = mode;
  isDrawerOpen.value = true;
};

const handleSelectItem = (item: Item) => {
  selectedItem.value = item;
  drawerMode.value = 'view';
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
  await auth.ensureUsername();
  unlistenRemoteCatchup = await listen('remote-catchup', () => handleRefreshItems());
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

    const normalized = nextUsername?.trim().toLowerCase();
    if (!normalized || normalized === 'unknown') {
      await auth.ensureUsername();
      return;
    }
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
        Welcome {{ displayUsername }}
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
          @refresh="handleRefreshItems"
          @open-drawer="openDrawer"
          @toggle-fullscreen="toggleFullscreen"
        />
      </template>

      <main class="content-body" :class="{ 'fullscreen-mode': isFullscreen }">
        <section class="plot-container">
          <button 
            v-if="isFullscreen" 
            class="exit-fullscreen-btn" 
            @click="toggleFullscreen"
          >
            Exit Fullscreen (Esc)
          </button>
          
          <ScatterPlot
            :items="items"
            v-model:range="selectedRange"
            v-model:y-field="selectedYField"
            v-model:color-field="selectedColorField"
            v-model:radius-field="selectedRadiusField"
            class="plot-component"
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
  background: linear-gradient(to bottom right, #f1f5f9, #f8fafc, #eff6ff);
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
  background: #f8fafc;
}

/* --- Plot Area (Added padding for axis spacing) --- */
.plot-container {
  flex: 1;
  position: relative;
  border-radius: 1rem;
  border: 1px solid #e2e8f0;
  background-color: #fff;
  /* 点が軸に近すぎないように内側に余白を追加 */
  padding: 1.5rem 1.5rem 1rem 1rem; 
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.05);
  display: flex;
  flex-direction: column;
}

.plot-component {
  flex: 1;
  width: 100%;
  height: 100%;
  min-height: 0;
}

.exit-fullscreen-btn {
  position: absolute;
  top: 1rem;
  right: 1rem;
  z-index: 100;
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(8px);
  border: 1px solid #e2e8f0;
  padding: 0.5rem 1rem;
  border-radius: 0.625rem;
  font-size: 0.75rem;
  font-weight: 600;
  color: #475569;
  cursor: pointer;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  transition: all 0.2s;
}

.exit-fullscreen-btn:hover {
  background: #fff;
  transform: translateY(-1px);
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
  z-index: 1000;
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