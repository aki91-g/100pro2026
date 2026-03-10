<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue';
import type { Item } from '@/types/item';
import type {
  GraphAxisField,
  GraphConfig,
  GraphGroup,
  GraphMarker,
  GraphTimeRangeKey,
  GraphVisualField,
} from '@/types/graph';
import { useGraph } from '@/composables/useGraph';

const props = defineProps<{
  items: Item[];
}>();

const graphConfig: GraphConfig = {
  padding: { top: 28, right: 28, bottom: 64, left: 76 },
  defaultMotivation: 5,
  defaultRadius: 8,
  minRadius: 6,
  maxRadius: 18,
  collisionPadding: 2,
  groupDistance: 26,
  tickCount: 90,
  timeRanges: {
    '1d': 24 * 60 * 60 * 1000,
    '3d': 3 * 24 * 60 * 60 * 1000,
    '1w': 7 * 24 * 60 * 60 * 1000,
    '2w': 14 * 24 * 60 * 60 * 1000,
    '1m': 30 * 24 * 60 * 60 * 1000,
  },
  palette: ['#cbd5e1', '#94a3b8', '#60a5fa', '#2563eb', '#1d4ed8'],
  neutralColor: '#94a3b8',
  overdueShade: 'rgba(248, 113, 113, 0.12)',
};

const timeRangeOptions: Array<{ value: GraphTimeRangeKey; label: string }> = [
  { value: '1d', label: '1 day' },
  { value: '3d', label: '3 days' },
  { value: '1w', label: '1 week' },
  { value: '2w', label: '2 weeks' },
  { value: '1m', label: '1 month' },
];

const axisFieldOptions: Array<{ value: GraphAxisField; label: string }> = [
  { value: 'duration_minutes', label: 'Duration' },
  { value: 'motivation', label: 'Motivation' },
  { value: 'status', label: 'Status' },
];

const visualFieldOptions: Array<{ value: GraphVisualField; label: string }> = [
  { value: 'none', label: 'None' },
  { value: 'duration_minutes', label: 'Duration' },
  { value: 'motivation', label: 'Motivation' },
  { value: 'status', label: 'Status' },
];

const containerRef = ref<HTMLDivElement | null>(null);
const hoveredGroup = ref<GraphGroup | null>(null);

const {
  graphGroups,
  warnings,
  debugStats,
  layout,
  xTicks,
  yTicks,
  selectedRange,
  selectedYField,
  selectedColorField,
  selectedRadiusField,
  groupingEnabled,
  setDimensions,
  updateData,
  refreshNow,
  destroy,
} = useGraph(graphConfig);

let resizeObserver: ResizeObserver | null = null;
let refreshTimer: number | null = null;

function updateViewportSize(): void {
  const container = containerRef.value;
  if (!container) return;
  const { width, height } = container.getBoundingClientRect();
  setDimensions(width, height);
}

function trianglePath(group: GraphGroup): string {
  const size = Math.max(group.radius, 10);
  if (group.marker === 'triangle-left') {
    return `M ${group.point.x - size} ${group.point.y} L ${group.point.x + size * 0.7} ${group.point.y - size * 0.78} L ${group.point.x + size * 0.7} ${group.point.y + size * 0.78} Z`;
  }

  return `M ${group.point.x + size} ${group.point.y} L ${group.point.x - size * 0.7} ${group.point.y - size * 0.78} L ${group.point.x - size * 0.7} ${group.point.y + size * 0.78} Z`;
}

function tooltipStyle(group: GraphGroup): Record<string, string> {
  const offsetLeft = Math.min(group.point.x + 18, Math.max(layout.value.width - 220, 0));
  const offsetTop = Math.max(group.point.y - 96, 12);
  return {
    left: `${offsetLeft}px`,
    top: `${offsetTop}px`,
  };
}

function formatDue(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString();
}

function formatMetric(value: number | null): string {
  return value === null ? 'None' : String(value);
}

function shortId(value: string): string {
  return value.slice(0, 8);
}

function handleGroupEnter(group: GraphGroup): void {
  hoveredGroup.value = group;
}

function handleGroupLeave(): void {
  hoveredGroup.value = null;
}

function formatRangeHint(range: GraphTimeRangeKey): string {
  return `Showing ${range} around now`;
}

function markerTextAnchor(marker: GraphMarker): 'start' | 'middle' | 'end' {
  if (marker === 'triangle-left') return 'start';
  if (marker === 'triangle-right') return 'end';
  return 'middle';
}

watch(
  () => props.items,
  (items) => {
    updateData(items);
  },
  { deep: true, immediate: true },
);

watch([selectedRange, selectedYField, selectedColorField, selectedRadiusField], () => {
  hoveredGroup.value = null;
  refreshNow();
});

onMounted(() => {
  updateViewportSize();
  resizeObserver = new ResizeObserver(() => {
    updateViewportSize();
  });

  if (containerRef.value) {
    resizeObserver.observe(containerRef.value);
  }

  refreshTimer = window.setInterval(() => {
    refreshNow();
  }, 5 * 60 * 1000);
});

onUnmounted(() => {
  resizeObserver?.disconnect();
  if (refreshTimer !== null) {
    window.clearInterval(refreshTimer);
  }
  destroy();
});
</script>

<template>
  <section class="scatter-plot card">
    <header class="scatter-plot__header">
      <div>
        <h2>Task Field Plot</h2>
        <p>Due date stays fixed on the X-axis. Y, color, and radius can be reassigned without changing the item schema.</p>
      </div>
      <p class="scatter-plot__range-hint">{{ formatRangeHint(selectedRange) }}</p>
    </header>

    <div class="scatter-plot__controls">
      <label>
        <span>Window</span>
        <select v-model="selectedRange">
          <option v-for="option in timeRangeOptions" :key="option.value" :value="option.value">
            {{ option.label }}
          </option>
        </select>
      </label>

      <label>
        <span>Y Axis</span>
        <select v-model="selectedYField">
          <option v-for="option in axisFieldOptions" :key="option.value" :value="option.value">
            {{ option.label }}
          </option>
        </select>
      </label>

      <label>
        <span>Color</span>
        <select v-model="selectedColorField">
          <option v-for="option in visualFieldOptions" :key="option.value" :value="option.value">
            {{ option.label }}
          </option>
        </select>
      </label>

      <label>
        <span>Radius</span>
        <select v-model="selectedRadiusField">
          <option v-for="option in visualFieldOptions" :key="option.value" :value="option.value">
            {{ option.label }}
          </option>
        </select>
      </label>

      <label>
        <span>Grouping</span>
        <select v-model="groupingEnabled">
          <option :value="false">Off (show each dot)</option>
          <option :value="true">On (aggregate nearby dots)</option>
        </select>
      </label>
    </div>

    <div v-if="warnings.length > 0" class="scatter-plot__warnings">
      <p v-for="warning in warnings" :key="warning">{{ warning }}</p>
    </div>

    <div class="scatter-plot__debug">
      <span>Input <strong>{{ debugStats.inputItems }}</strong></span>
      <span>Visible <strong>{{ debugStats.visibleItems }}</strong></span>
      <span>Plotted <strong>{{ debugStats.plottedItems }}</strong></span>
      <span class="scatter-plot__debug-skipped" :class="{ 'scatter-plot__debug-skipped--warn': debugStats.invalidDueCount > 0 }">
        Skipped <strong>{{ debugStats.invalidDueCount }}</strong>
      </span>
    </div>

    <div ref="containerRef" class="scatter-plot__stage">
      <svg
        v-if="layout.width > 0 && layout.height > 0"
        class="scatter-plot__svg"
        :viewBox="`0 0 ${layout.width} ${layout.height}`"
        preserveAspectRatio="none"
      >
        <rect
          v-if="layout.originX > layout.plotLeft"
          :x="layout.plotLeft"
          :y="layout.plotTop"
          :width="layout.originX - layout.plotLeft"
          :height="layout.innerHeight"
          :fill="graphConfig.overdueShade"
          rx="16"
        />

        <g class="scatter-plot__grid">
          <line
            v-for="tick in yTicks"
            :key="`y-${tick.position}`"
            :x1="layout.plotLeft"
            :x2="layout.plotRight"
            :y1="tick.position"
            :y2="tick.position"
          />

          <line
            v-for="tick in xTicks"
            :key="`x-${tick.position}`"
            :x1="tick.position"
            :x2="tick.position"
            :y1="layout.plotTop"
            :y2="layout.plotBottom"
          />
        </g>

        <line
          class="scatter-plot__origin"
          :x1="layout.originX"
          :x2="layout.originX"
          :y1="layout.plotTop"
          :y2="layout.plotBottom"
        />

        <line
          class="scatter-plot__axis"
          :x1="layout.plotLeft"
          :x2="layout.plotRight"
          :y1="layout.plotBottom"
          :y2="layout.plotBottom"
        />

        <g class="scatter-plot__labels">
          <text
            v-for="tick in yTicks"
            :key="`yl-${tick.position}`"
            :x="layout.plotLeft - 12"
            :y="tick.position + 4"
            text-anchor="end"
          >
            {{ tick.label }}
          </text>

          <text
            v-for="tick in xTicks"
            :key="`xl-${tick.position}`"
            :x="tick.position"
            :y="layout.plotBottom + 24"
            text-anchor="middle"
          >
            {{ tick.label }}
          </text>

          <text :x="layout.plotLeft" :y="layout.plotTop - 8">Overdue</text>
          <text :x="layout.plotRight" :y="layout.plotTop - 8" text-anchor="end">Upcoming</text>
        </g>

        <g class="scatter-plot__nodes">
          <g
            v-for="group in graphGroups"
            :key="group.key"
            class="scatter-plot__node"
            @mouseenter="handleGroupEnter(group)"
            @mouseleave="handleGroupLeave"
          >
            <circle
              v-if="group.marker === 'circle'"
              :cx="group.point.x"
              :cy="group.point.y"
              :r="group.radius"
              :fill="group.color"
            />
            <path
              v-else
              :d="trianglePath(group)"
              :fill="group.color"
            />

            <text
              v-if="group.label"
              :x="group.point.x"
              :y="group.point.y + 4"
              :text-anchor="markerTextAnchor(group.marker)"
              class="scatter-plot__count"
            >
              {{ group.label }}
            </text>
          </g>
        </g>
      </svg>

      <div v-if="hoveredGroup" :style="tooltipStyle(hoveredGroup)" class="scatter-plot__tooltip">
        <strong>{{ hoveredGroup.representative.title }}</strong>
        <span>ID {{ shortId(hoveredGroup.representative.id) }}</span>
        <span>Due {{ formatDue(hoveredGroup.representative.due) }}</span>
        <span>Status {{ hoveredGroup.representative.status }}</span>
        <span>Motivation {{ formatMetric(hoveredGroup.representative.motivation) }}</span>
        <span>Duration {{ formatMetric(hoveredGroup.representative.duration_minutes) }}</span>
        <span v-if="hoveredGroup.items.length > 1">Grouped {{ hoveredGroup.items.length }} items</span>
      </div>
    </div>
  </section>
</template>

<style scoped>
.scatter-plot {
  display: grid;
  gap: 1rem;
}

.scatter-plot__header {
  display: flex;
  justify-content: space-between;
  gap: 1rem;
  align-items: flex-start;
}

.scatter-plot__header h2 {
  margin: 0 0 0.35rem;
  font-size: 1.1rem;
}

.scatter-plot__header p {
  margin: 0;
  color: #64748b;
  line-height: 1.5;
}

.scatter-plot__range-hint {
  white-space: nowrap;
  font-size: 0.9rem;
}

.scatter-plot__controls {
  display: grid;
  grid-template-columns: repeat(5, minmax(0, 1fr));
  gap: 0.75rem;
}

.scatter-plot__controls label {
  display: grid;
  gap: 0.35rem;
  font-size: 0.85rem;
  color: #475569;
}

.scatter-plot__controls select {
  border: 1px solid #cbd5e1;
  border-radius: 10px;
  padding: 0.6rem 0.7rem;
  background: white;
  color: #0f172a;
}

.scatter-plot__warnings {
  border: 1px solid #fecaca;
  background: #fff1f2;
  color: #b91c1c;
  border-radius: 12px;
  padding: 0.75rem 0.9rem;
}

.scatter-plot__warnings p {
  margin: 0;
}

.scatter-plot__debug {
  display: flex;
  gap: 1.25rem;
  padding: 0.55rem 0.9rem;
  border-radius: 10px;
  background: #f1f5f9;
  border: 1px solid #e2e8f0;
  font-size: 0.8rem;
  color: #475569;
}

.scatter-plot__debug strong {
  font-variant-numeric: tabular-nums;
  color: #0f172a;
}

.scatter-plot__debug-skipped--warn strong {
  color: #b91c1c;
}

.scatter-plot__stage {
  position: relative;
  min-height: 440px;
  border-radius: 18px;
  overflow: hidden;
  background:
    radial-gradient(circle at top left, rgba(96, 165, 250, 0.12), transparent 35%),
    linear-gradient(180deg, #ffffff 0%, #f8fafc 100%);
  border: 1px solid #e2e8f0;
}

.scatter-plot__svg {
  display: block;
  width: 100%;
  height: 100%;
  min-height: 440px;
}

.scatter-plot__grid line {
  stroke: rgba(148, 163, 184, 0.28);
  stroke-dasharray: 4 6;
}

.scatter-plot__axis,
.scatter-plot__origin {
  stroke: #94a3b8;
  stroke-width: 1.5;
}

.scatter-plot__labels text {
  fill: #64748b;
  font-size: 11px;
}

.scatter-plot__node {
  cursor: pointer;
}

.scatter-plot__node circle,
.scatter-plot__node path {
  stroke: rgba(255, 255, 255, 0.92);
  stroke-width: 2;
}

.scatter-plot__count {
  fill: white;
  font-size: 11px;
  font-weight: 700;
  pointer-events: none;
}

.scatter-plot__tooltip {
  position: absolute;
  z-index: 2;
  display: grid;
  gap: 0.25rem;
  min-width: 190px;
  padding: 0.8rem 0.9rem;
  border-radius: 12px;
  background: rgba(15, 23, 42, 0.92);
  color: white;
  box-shadow: 0 18px 40px rgba(15, 23, 42, 0.2);
  pointer-events: none;
}

.scatter-plot__tooltip span {
  font-size: 0.8rem;
  color: rgba(255, 255, 255, 0.84);
}

@media (max-width: 900px) {
  .scatter-plot__header {
    flex-direction: column;
  }

  .scatter-plot__controls {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 640px) {
  .scatter-plot__controls {
    grid-template-columns: 1fr;
  }
}
</style>