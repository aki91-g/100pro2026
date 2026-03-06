<script setup lang="ts">
defineProps<{
  visible: boolean;
  isAuthenticated: boolean;
  username: string | null;
}>();

const emit = defineEmits<{
  seed: [];
  reset: [];
  migrate: [];
}>();
</script>

<template>
  <section class="card debug-section" v-if="visible">
    <h2 style="color: #d32f2f;">🛠 Debug Tools</h2>
    <p class="description">These tools are only visible in development builds.</p>
    <p class="description" v-if="isAuthenticated" style="color: #42b883;">
      ✓ Logged in as: <strong>{{ username || 'User' }}</strong>
    </p>
    <p class="description" v-else style="color: #f57c00;">
      ⚠ Login required to use debug tools
    </p>
    <div class="input-group">
      <button @click="emit('seed')" :disabled="!isAuthenticated">Seed Demo Data</button>
      <button @click="emit('reset')" :disabled="!isAuthenticated" class="btn-danger">Wipe My Data</button>
    </div>
    <div class="input-group" style="margin-top: 0.5rem;">
      <button @click="emit('migrate')" :disabled="!isAuthenticated" class="btn-migration">
        🔄 Claim Orphaned Items
      </button>
    </div>
    <p class="description" style="font-size: 0.75rem; color: #666; margin-top: 0.5rem;">
      💡 Tip: "Claim Orphaned Items" assigns items with NULL user_id to your account.
    </p>
  </section>
</template>

<style scoped>
.card { background: #f8f9fa; border-radius: 12px; padding: 1.5rem; margin-bottom: 1.5rem; border: 1px solid #eee; }
.debug-section { border: 2px dashed #ffcdd2; background: #fff9f9; }
.description { color: #666; font-size: 0.9rem; margin-bottom: 1rem; }
.input-group { display: flex; gap: 10px; margin-bottom: 1rem; }
button { background: #34495e; color: white; border: none; padding: 0.6rem 1.2rem; border-radius: 6px; cursor: pointer; }
button:hover { background: #41b883; }
button:disabled { background: #999; cursor: not-allowed; }
.btn-danger { background: #e53935; }
.btn-danger:hover { background: #c62828; }
.btn-migration { background: #3498db; }
.btn-migration:hover { background: #2980b9; }
.btn-migration:disabled { background: #999; }
</style>