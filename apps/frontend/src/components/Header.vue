<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import { useSettings } from '@/composables/useSettings';

const props = defineProps<{
  displayUsername: string;
  isSyncing: boolean;
  isGuest: boolean;
}>();

const emit = defineEmits<{
  logout: [];
  'show-thanks': [];
  'show-help': [];
}>();

const { theme, language, toggleTheme, toggleLanguage, t } = useSettings();

const isMenuOpen = ref(false);
const menuRootRef = ref<HTMLElement | null>(null);
const menuPanelRef = ref<HTMLElement | null>(null);

const avatarLabel = computed(() => {
  const trimmed = props.displayUsername?.trim();
  if (!trimmed) return 'U';
  return trimmed.slice(0, 1).toUpperCase();
});

const syncStatusLabel = computed(() => {
  if (props.isGuest) return t('guestLocalMode');
  return props.isSyncing ? t('dbSyncing') : t('dbIdle');
});

const themeLabel = computed(() => (theme.value === 'light' ? t('light') : t('dark')));
const languageLabel = computed(() => (language.value === 'en' ? t('english') : t('japanese')));

function toggleMenu(): void {
  isMenuOpen.value = !isMenuOpen.value;
}

function closeMenu(): void {
  isMenuOpen.value = false;
}

function focusMenuItem(index: number): void {
  const menuPanel = menuPanelRef.value;
  if (!menuPanel) return;

  const items = Array.from(menuPanel.querySelectorAll<HTMLElement>('[role="menuitem"]'));
  if (items.length === 0) return;

  const wrappedIndex = (index + items.length) % items.length;
  items[wrappedIndex]?.focus();
}

function handleMenuKeydown(event: KeyboardEvent): void {
  if (!isMenuOpen.value) return;

  const menuPanel = menuPanelRef.value;
  if (!menuPanel) return;

  const items = Array.from(menuPanel.querySelectorAll<HTMLElement>('[role="menuitem"]'));
  if (items.length === 0) return;

  const activeIndex = items.findIndex((item) => item === document.activeElement);

  if (event.key === 'ArrowDown') {
    event.preventDefault();
    focusMenuItem(activeIndex < 0 ? 0 : activeIndex + 1);
    return;
  }

  if (event.key === 'ArrowUp') {
    event.preventDefault();
    focusMenuItem(activeIndex < 0 ? items.length - 1 : activeIndex - 1);
    return;
  }

  if (event.key === 'Escape') {
    event.preventDefault();
    closeMenu();
  }
}

function handleLogout(): void {
  closeMenu();
  emit('logout');
}

function handleDocumentClick(event: MouseEvent): void {
  if (!menuRootRef.value) return;
  const target = event.target as Node | null;
  if (!target) return;
  if (!menuRootRef.value.contains(target)) {
    closeMenu();
  }
}

onMounted(() => {
  document.addEventListener('click', handleDocumentClick);
});

onUnmounted(() => {
  document.removeEventListener('click', handleDocumentClick);
});

watch(isMenuOpen, async (open) => {
  if (!open) return;
  await nextTick();
  focusMenuItem(0);
});
</script>

<template>
  <header class="app-header">
    <div class="header-content">
      <div class="branding">
        <h1 class="title">{{ t('appTitle') }}</h1>
        <button
          type="button"
          class="version-badge" 
          :title="t('specialThanks')"
          :aria-label="t('specialThanks')"
          @click="emit('show-thanks')"
        >
          <span class="gradient-text">100 program v9</span>
        </button>
      </div>

      <div class="user-actions">
        <button 
          class="help-circle" 
          :title="t('helpTitle')"
          :aria-label="t('helpTitle')"
          @click="emit('show-help')"
        >
          ?
        </button>

        <div class="profile-menu" ref="menuRootRef">
          <button
            type="button"
            class="profile-trigger"
            :aria-expanded="isMenuOpen"
            aria-haspopup="menu"
            @click.stop="toggleMenu"
          >
            <span class="avatar">{{ avatarLabel }}</span>
            <span class="username">{{ displayUsername }}</span>
            <span v-if="isGuest" class="guest-badge">{{ t('guestMode') }}</span>
          </button>

          <transition name="menu-fade">
            <div v-if="isMenuOpen" ref="menuPanelRef" class="menu-panel" role="menu" @keydown="handleMenuKeydown" @click.stop>
              <button type="button" class="menu-item" role="menuitem" @click="toggleTheme">
                <span class="menu-label">{{ t('theme') }}</span>
                <span class="menu-value">{{ themeLabel }}</span>
              </button>

              <button type="button" class="menu-item" role="menuitem" @click="toggleLanguage">
                <span class="menu-label">{{ t('language') }}</span>
                <span class="menu-value">{{ languageLabel }}</span>
              </button>

              <div class="menu-item menu-status" :class="{ 'is-syncing': isSyncing && !isGuest }">
                <span class="menu-label">{{ t('syncStatus') }}</span>
                <span class="status-value">
                  <span class="dot" />
                  {{ syncStatusLabel }}
                </span>
              </div>

              <button type="button" class="menu-item logout-item" role="menuitem" @click="handleLogout">
                {{ t('logout') }}
              </button>
            </div>
          </transition>
        </div>
      </div>
    </div>
  </header>
</template>

<style scoped>
.app-header {
  position: relative;
  z-index: var(--z-header);
  border-radius: 1rem;
  border: 1px solid var(--tg-border-muted);
  background-color: var(--tg-surface-translucent);
  padding: 0.75rem 1rem;
  box-shadow: var(--tg-shadow-soft);
  backdrop-filter: blur(4px);
}

.header-content {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
}

.branding { display: flex; align-items: center; gap: 0.75rem; }

.title {
  font-size: 1.25rem;
  font-weight: 600;
  letter-spacing: -0.025em;
  color: var(--tg-text-strong);
  margin: 0;
}

.version-badge {
  appearance: none;
  border-radius: 9999px;
  border: 1px solid #e2e8f0;
  background-color: var(--badge-bg);
  padding: 0.25rem 0.75rem;
  font-size: 0.875rem;
  font-weight: 600;
  cursor: pointer;
  transition: transform 0.2s, box-shadow 0.2s;
  user-select: none; /* テキスト選択を防いでボタン感を出す */
}

.version-badge:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(168, 85, 247, 0.15);
  background-color: var(--badge-bg);
}

.gradient-text {
  background: linear-gradient(to right, #ef4444, #a855f7, #3b82f6);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}

.user-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.75rem;
}

.help-circle {
  width: 26px;
  height: 26px;
  border-radius: 50%;
  border: 1px solid var(--tg-border-default);
  background: var(--tg-surface);
  color: var(--tg-text-muted);
  font-size: 0.8rem;
  font-weight: 700;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.help-circle:hover {
  background: var(--tg-surface-raised);
  color: var(--tg-text-default);
  border-color: var(--tg-border-strong);
  transform: scale(1.05);
}

.profile-menu {
  position: relative;
  z-index: calc(var(--z-header) + 10);
  min-width: 0;
}

.profile-trigger {
  display: inline-flex;
  align-items: center;
  min-width: 0;
  gap: 0.625rem;
  border-radius: 9999px;
  border: 1px solid var(--tg-border-default);
  background-color: var(--tg-surface);
  padding: 0.375rem 0.625rem;
  cursor: pointer;
  transition: transform 0.2s, box-shadow 0.2s, border-color 0.2s;
}

.profile-trigger:hover {
  transform: translateY(-1px);
  box-shadow: var(--tg-shadow-soft);
  border-color: var(--tg-border-strong);
}

.avatar {
  width: 1.75rem;
  height: 1.75rem;
  border-radius: 50%;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 0.8rem;
  font-weight: 700;
  color: #ffffff;
  background: linear-gradient(135deg, #ef4444, #a855f7, #3b82f6);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.25);
}

.dot {
  height: 0.5rem;
  width: 0.5rem;
  border-radius: 9999px;
  background-color: #94a3b8;
}

.is-syncing .dot {
  background-color: #3b82f6;
  animation: pulse 2s infinite;
}

.username {
  min-width: 0;
  max-width: 12rem;
  font-weight: 500;
  color: var(--tg-text-strong);
  text-overflow: ellipsis;
  white-space: nowrap;
  overflow: hidden;
}

.guest-badge {
  border-radius: 9999px;
  border: 1px solid #fbbf24;
  background: #fef3c7;
  color: #92400e;
  padding: 0.15rem 0.5rem;
  font-size: 0.7rem;
  font-weight: 700;
}

.menu-panel {
  position: absolute;
  top: calc(100% + 0.55rem);
  right: 0;
  min-width: 18rem;
  border-radius: 0.85rem;
  border: 1px solid var(--tg-border-default);
  background: var(--tg-surface);
  box-shadow: var(--tg-shadow-elevated);
  padding: 0.45rem;
  z-index: var(--z-dropdown);
}

.menu-item {
  width: 100%;
  border: 0;
  background: transparent;
  border-radius: 0.6rem;
  color: var(--tg-text-default);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  padding: 0.625rem 0.75rem;
  font-size: 0.82rem;
  text-align: left;
  transition: background-color 0.2s, color 0.2s;
}

.menu-item:hover {
  background-color: var(--tg-surface-raised);
}

.menu-label {
  font-weight: 600;
}

.menu-value {
  color: var(--tg-text-muted);
}

.menu-status {
  cursor: default;
}

.menu-status .status-value {
  display: inline-flex;
  align-items: center;
  gap: 0.45rem;
  color: var(--tg-text-muted);
}

.menu-status.is-syncing .status-value {
  color: #1d4ed8;
}

.logout-item {
  color: #dc2626;
  font-weight: 700;
}

.logout-item:hover {
  background-color: var(--bg-danger-hover, #fef2f2);
}

.menu-fade-enter-active,
.menu-fade-leave-active {
  transition: opacity 0.18s ease, transform 0.18s ease;
  transform-origin: top right;
}

.menu-fade-enter-from,
.menu-fade-leave-to {
  opacity: 0;
  transform: translateY(-6px) scale(0.98);
}

@media (max-width: 640px) {
  .header-content {
    align-items: flex-start;
  }

  .menu-panel {
    right: 0;
    left: auto;
    min-width: min(18rem, 92vw);
  }
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: .5; }
}
</style>