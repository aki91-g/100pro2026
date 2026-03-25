<script setup lang="ts">
import type { GraphAxisField, GraphTimeRangeKey, GraphVisualField, SelectOption } from '@/types/graph';
import { useSettings } from '@/composables/useSettings';

const { t } = useSettings();

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
  'openDrawer': [mode: 'tasks' | 'create'];
}>();
</script>

<template>
  <section class="controls-bar">
    <div class="controls-grid">
      <div class="input-group">
        <label>{{ t('controlWindow') }}</label>
        <select :value="range" @change="emit('update:range', ($event.target as HTMLSelectElement).value as GraphTimeRangeKey)">
          <option v-for="o in rangeOptions" :key="o.value" :value="o.value">{{ o.label }}</option>
        </select>
      </div>

      <div class="input-group">
        <label>{{ t('controlYAxis') }}</label>
        <select :value="yField" @change="emit('update:yField', ($event.target as HTMLSelectElement).value as GraphAxisField)">
          <option v-for="o in axisOptions" :key="o.value" :value="o.value">{{ o.label }}</option>
        </select>
      </div>

      <div class="input-group">
        <label>{{ t('controlColor') }}</label>
        <select :value="colorField" @change="emit('update:colorField', ($event.target as HTMLSelectElement).value as GraphVisualField)">
          <option v-for="o in visualOptions" :key="o.value" :value="o.value">{{ o.label }}</option>
        </select>
      </div>

      <div class="input-group">
        <label>{{ t('controlRadius') }}</label>
        <select :value="radiusField" @change="emit('update:radiusField', ($event.target as HTMLSelectElement).value as GraphVisualField)">
          <option v-for="o in visualOptions" :key="o.value" :value="o.value">{{ o.label }}</option>
        </select>
      </div>
    </div>

    <div class="button-group">
      <button class="secondary-btn" @click="emit('openDrawer', 'tasks')">{{ t('list') }}</button>
      <button class="primary-btn" @click="emit('openDrawer', 'create')">{{ t('new') }}</button>
    </div>
  </section>
</template>

<style scoped>
.controls-bar {
  margin-top: 0.75rem;
  border-radius: 1rem;
  border: 1px solid var(--tg-border-muted);
  background-color: var(--tg-surface-translucent);
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
  color: var(--text-muted);
  margin-bottom: 2px;
}

.input-group select {
  border-radius: 0.5rem;
  border: 1px solid var(--tg-border-default);
  background-color: var(--bg-primary);
  padding: 0.375rem 0.75rem;
  font-size: 0.875rem;
  color: var(--text-primary);
  outline: none;
  cursor: pointer;
}

.button-group { 
  display: flex; 
  gap: 0.5rem; 
  align-items: center;
}

.primary-btn {
  background: linear-gradient(135deg, #ef4444 0%, #a855f7 50%, #3b82f6 100%);
  color: #fff;
  border: none;
  padding: 0.375rem 0.75rem;
  border-radius: 0.5rem;
  font-weight: 600;
  cursor: pointer;
  transition: transform 0.2s ease, filter 0.2s ease;
}

.primary-btn:hover {
  transform: translateY(-1px);
  filter: brightness(1.06);
}

.secondary-btn {
  background-color: var(--bg-primary);
  border: 1px solid var(--tg-border-default);
  padding: 0.375rem 0.75rem;
  border-radius: 0.5rem;
  color: var(--text-primary);
  font-size: 0.875rem;
  cursor: pointer;
  transition: all 0.2s;
}

.secondary-btn:hover {
  background-color: var(--bg-secondary);
  border-color: var(--tg-border-strong);
}
</style>