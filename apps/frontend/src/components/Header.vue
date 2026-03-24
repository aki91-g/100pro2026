<script setup lang="ts">
defineProps<{
  displayUsername: string;
  isSyncing: boolean;
}>();

const emit = defineEmits<{
  logout: [];
  'show-thanks': [];
  'show-help': [];
}>();
</script>

<template>
  <header class="app-header">
    <div class="header-content">
      <div class="branding">
        <h1 class="title">TaskGraph</h1>
        <div 
          class="version-badge" 
          role="button" 
          title="Special Thanks"
          @click="emit('show-thanks')"
        >
          <span class="gradient-text">100 program v9</span>
        </div>
      </div>

      <div class="user-actions">
        <button 
          class="help-circle" 
          title="How to use"
          @click="emit('show-help')"
        >
          ?
        </button>

        <span class="status-indicator" :class="{ 'is-syncing': isSyncing }">
          <span class="dot" />
          {{ isSyncing ? 'Database syncing...' : 'Database idle' }}
        </span>

        <div class="profile-chip">
          <span class="username">{{ displayUsername }}</span>
          <button type="button" class="logout-btn" @click="emit('logout')">
            Logout
          </button>
        </div>
      </div>
    </div>
  </header>
</template>

<style scoped>
.app-header {
  border-radius: 1rem;
  border: 1px solid rgba(226, 232, 240, 0.8);
  background-color: rgba(255, 255, 255, 0.9);
  padding: 0.75rem 1rem;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
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
  color: #020617;
  margin: 0;
}

.version-badge {
  border-radius: 9999px;
  border: 1px solid #e2e8f0;
  background-color: #fff;
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
  background-color: #fafafa;
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
  border: 1px solid #cbd5e1;
  background: white;
  color: #64748b;
  font-size: 0.8rem;
  font-weight: 700;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.help-circle:hover {
  background: #f8fafc;
  color: #1e293b;
  border-color: #94a3b8;
  transform: scale(1.05);
}

.status-indicator {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  border-radius: 9999px;
  border: 1px solid #cbd5e1;
  background-color: #f1f5f9;
  padding: 0.25rem 0.75rem;
  font-size: 0.75rem;
  font-weight: 500;
  color: #334155;
}

.status-indicator.is-syncing {
  border-color: #93c5fd;
  background-color: #eff6ff;
  color: #1d4ed8;
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

.profile-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.75rem;
  border-radius: 9999px;
  border: 1px solid #e2e8f0;
  background-color: #fff;
  padding: 0.375rem 0.75rem;
  font-size: 0.875rem;
}

.username { font-weight: 500; color: #0f172a; }

.logout-btn {
  border-radius: 0.375rem;
  border: 1px solid #cbd5e1;
  background-color: #fff;
  padding: 0.25rem 0.625rem;
  font-size: 0.75rem;
  cursor: pointer;
  transition: all 0.2s;
}

.logout-btn:hover {
  background-color: #fef2f2;
  border-color: #fecaca;
  color: #ef4444;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: .5; }
}
</style>