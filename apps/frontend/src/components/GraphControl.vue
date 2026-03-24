<script setup lang="ts">
import type { GraphAxisField, GraphTimeRangeKey, GraphVisualField, SelectOption } from '@/types/graph';

defineProps<{
  range: GraphTimeRangeKey;
  yField: GraphAxisField;
  colorField: GraphVisualField;
  radiusField: GraphVisualField;
  rangeOptions: SelectOption<GraphTimeRangeKey>[];
  axisOptions: SelectOption<GraphAxisField>[];
  visualOptions: SelectOption<GraphVisualField>[];
}>();

const emit = defineEmits<{
  'update:range': [val: GraphTimeRangeKey];
  'update:yField': [val: GraphAxisField];
  'update:colorField': [val: GraphVisualField];
  'update:radiusField': [val: GraphVisualField];
  'refresh': [];
  'openDrawer': [mode: 'tasks' | 'create'];
  'toggle-fullscreen': [];
}>();
</script>

<template>
  <section class="controls-bar">
    <div class="controls-grid">
      <div class="input-group">
        <label>Window</label>
        <select :value="range" @change="emit('update:range', ($event.target as HTMLSelectElement).value as GraphTimeRangeKey)">
          <option v-for="o in rangeOptions" :key="o.value" :value="o.value">{{ o.label }}</option>
        </select>
      </div>

      <div class="input-group">
        <label>Y-Axis</label>
        <select :value="yField" @change="emit('update:yField', ($event.target as HTMLSelectElement).value as GraphAxisField)">
          <option v-for="o in axisOptions" :key="o.value" :value="o.value">{{ o.label }}</option>
        </select>
      </div>

      <div class="input-group">
        <label>Color</label>
        <select :value="colorField" @change="emit('update:colorField', ($event.target as HTMLSelectElement).value as GraphVisualField)">
          <option v-for="o in visualOptions" :key="o.value" :value="o.value">{{ o.label }}</option>
        </select>
      </div>

      <div class="input-group">
        <label>Radius</label>
        <select :value="radiusField" @change="emit('update:radiusField', ($event.target as HTMLSelectElement).value as GraphVisualField)">
          <option v-for="o in visualOptions" :key="o.value" :value="o.value">{{ o.label }}</option>
        </select>
      </div>
    </div>

    <div class="button-group">
      <button class="secondary-btn" @click="emit('refresh')" title="Refresh Data">
        Refresh
      </button>

      <button class="secondary-btn flex-btn" @click="emit('toggle-fullscreen')" title="Maximize Graph">
        <svg class="icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8V4h4m8 0h4v4m0 8v4h-4m-8 0H4v-4" />
        </svg>
        Maximize
      </button>

      <button class="secondary-btn" @click="emit('openDrawer', 'tasks')">Tasks</button>
      <button class="primary-btn" @click="emit('openDrawer', 'create')">New Task</button>
    </div>
  </section>
</template>

<style scoped>
.controls-bar {
  margin-top: 0.75rem;
  border-radius: 1rem;
  border: 1px solid rgba(226, 232, 240, 0.8);
  background-color: rgba(255, 255, 255, 0.9);
  padding: 0.625rem 1rem;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  backdrop-filter: blur(4px);
}

.controls-grid { 
  display: flex; 
  flex: 1; 
  flex-wrap: wrap; 
  gap: 0.75rem; 
}

.input-group { 
  display: flex; 
  flex-direction: column; 
  min-width: 140px; 
}

.input-group label {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.18em;
  color: #94a3b8;
  margin-bottom: 2px;
}

.input-group select {
  border-radius: 0.5rem;
  border: 1px solid #cbd5e1;
  background-color: #fff;
  padding: 0.375rem 0.75rem;
  font-size: 0.875rem;
  outline: none;
  cursor: pointer;
}

.button-group { 
  display: flex; 
  gap: 0.5rem; 
  align-items: center;
}

.flex-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
}

.icon {
  width: 14px;
  height: 14px;
}

.primary-btn {
  background-color: #2563eb;
  color: #fff;
  border: none;
  padding: 0.375rem 0.75rem;
  border-radius: 0.5rem;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.2s;
}

.primary-btn:hover {
  background-color: #1d4ed8;
}

.secondary-btn {
  background-color: #fff;
  border: 1px solid #cbd5e1;
  padding: 0.375rem 0.75rem;
  border-radius: 0.5rem;
  color: #334155;
  font-size: 0.875rem;
  cursor: pointer;
  transition: all 0.2s;
}

.secondary-btn:hover {
  background-color: #f8fafc;
  border-color: #94a3b8;
}
</style>