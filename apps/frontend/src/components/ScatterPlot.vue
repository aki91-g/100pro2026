<script setup lang="ts">
import { computed, ref, watch, onMounted, onUnmounted, nextTick } from 'vue';
import type { Item } from '@/types/item';
import type {
  GraphAxisField,
  GraphConfig,
  GraphGroup,
  GraphTimeRangeKey,
  GraphVisualField,
} from '@/types/graph';
import { useGraph } from '@/composables/useGraph';

const props = defineProps<{
  items: Item[];
  showDebug?: boolean;
}>();

const emit = defineEmits<{
  (event: 'select-item', item: Item): void;
}>();

const selectedRangeModel = defineModel<GraphTimeRangeKey>('range', { default: '1w' });
const selectedYFieldModel = defineModel<GraphAxisField>('yField', { default: 'duration_minutes' });
const selectedColorFieldModel = defineModel<GraphVisualField>('colorField', { default: 'motivation' });
const selectedRadiusFieldModel = defineModel<GraphVisualField>('radiusField', { default: 'duration_minutes' });

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

const containerRef = ref<HTMLDivElement | null>(null);
const canvasRef = ref<HTMLCanvasElement | null>(null);
const clusterMenu = ref<{ visible: boolean; left: number; top: number; items: Item[] }>({
  visible: false,
  left: 0,
  top: 0,
  items: [],
});

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

groupingEnabled.value = true;

let resizeObserver: ResizeObserver | null = null;
let refreshTimer: number | null = null;
let animationFrame: number | null = null;
let sizeRetryFrame: number | null = null;

function clamp(value: number, minValue: number, maxValue: number): number {
  return Math.min(Math.max(value, minValue), maxValue);
}

const stageStyle = computed(() => ({
  width: '100%',
  height: '100%',
}));

function updateViewportSize(): void {
  const container = containerRef.value;
  if (!container) return;
  const { width, height } = container.getBoundingClientRect();
  if (width <= 1 || height <= 1) {
    if (sizeRetryFrame !== null) {
      window.cancelAnimationFrame(sizeRetryFrame);
    }
    sizeRetryFrame = window.requestAnimationFrame(() => {
      updateViewportSize();
    });
    return;
  }
  setDimensions(width, height);
}

function syncCanvasBackingStore(): CanvasRenderingContext2D | null {
  const canvas = canvasRef.value;
  if (!canvas) return null;

  const cssWidth = Math.max(layout.value.width, 1);
  const cssHeight = Math.max(layout.value.height, 1);
  const dpr = window.devicePixelRatio || 1;

  canvas.width = Math.floor(cssWidth * dpr);
  canvas.height = Math.floor(cssHeight * dpr);
  canvas.style.width = `${cssWidth}px`;
  canvas.style.height = `${cssHeight}px`;

  const ctx = canvas.getContext('2d');
  if (!ctx) return null;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  return ctx;
}

function drawTriangle(
  ctx: CanvasRenderingContext2D,
  marker: GraphGroup['marker'],
  centerX: number,
  centerY: number,
  radius: number,
): void {
  const size = Math.max(radius, 10);
  ctx.beginPath();
  if (marker === 'triangle-left') {
    ctx.moveTo(centerX - size, centerY);
    ctx.lineTo(centerX + size * 0.7, centerY - size * 0.78);
    ctx.lineTo(centerX + size * 0.7, centerY + size * 0.78);
  } else {
    ctx.moveTo(centerX + size, centerY);
    ctx.lineTo(centerX - size * 0.7, centerY - size * 0.78);
    ctx.lineTo(centerX - size * 0.7, centerY + size * 0.78);
  }
  ctx.closePath();
}

function drawScene(): void {
  const ctx = syncCanvasBackingStore();
  if (!ctx) return;

  const width = Math.max(layout.value.width, 1);
  const height = Math.max(layout.value.height, 1);

  ctx.clearRect(0, 0, width, height);

  if (layout.value.innerWidth <= 0 || layout.value.innerHeight <= 0) return;

  const { plotLeft, plotRight, plotTop, plotBottom, originX } = layout.value;

  // Zone styling
  ctx.fillStyle = 'rgba(255, 0, 0, 0.05)';
  ctx.fillRect(plotLeft, plotTop, Math.max(originX - plotLeft, 0), plotBottom - plotTop);

  ctx.fillStyle = 'rgba(239, 246, 255, 0.45)';
  ctx.fillRect(originX, plotTop, Math.max(plotRight - originX, 0), plotBottom - plotTop);

  // Grid lines
  ctx.strokeStyle = 'rgba(148, 163, 184, 0.2)';
  ctx.lineWidth = 1;
  ctx.setLineDash([4, 6]);
  for (const tick of yTicks.value) {
    ctx.beginPath();
    ctx.moveTo(plotLeft, tick.position);
    ctx.lineTo(plotRight, tick.position);
    ctx.stroke();
  }
  for (const tick of xTicks.value) {
    ctx.beginPath();
    ctx.moveTo(tick.position, plotTop);
    ctx.lineTo(tick.position, plotBottom);
    ctx.stroke();
  }

  // Now line
  ctx.strokeStyle = 'rgba(15, 23, 42, 0.45)';
  ctx.lineWidth = 1.4;
  ctx.setLineDash([6, 6]);
  ctx.beginPath();
  ctx.moveTo(originX, plotTop);
  ctx.lineTo(originX, plotBottom);
  ctx.stroke();
  ctx.setLineDash([]);

  // Axes labels
  ctx.fillStyle = '#64748b';
  ctx.font = '11px ui-sans-serif, system-ui, -apple-system, Segoe UI';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'top';
  for (const tick of xTicks.value) {
    ctx.fillText(tick.label, tick.position, plotBottom + 8);
  }
  ctx.textAlign = 'right';
  ctx.textBaseline = 'middle';
  for (const tick of yTicks.value) {
    ctx.fillText(tick.label, plotLeft - 10, tick.position);
  }

  ctx.textAlign = 'left';
  ctx.textBaseline = 'bottom';
  ctx.fillText('Past', plotLeft + 4, plotTop - 4);
  ctx.textAlign = 'right';
  ctx.fillText('Future', plotRight - 4, plotTop - 4);

  // Nodes and badges
  for (const group of graphGroups.value) {
    const radius = Math.max(group.radius, 6);

    ctx.save();
    ctx.fillStyle = group.color;
    ctx.strokeStyle = 'rgba(255,255,255,0.95)';
    ctx.lineWidth = 2;

    if (group.marker === 'circle') {
      ctx.beginPath();
      ctx.arc(group.point.x, group.point.y, radius, 0, Math.PI * 2);
    } else {
      drawTriangle(ctx, group.marker, group.point.x, group.point.y, radius);
    }

    ctx.fill();
    ctx.stroke();
    ctx.restore();

    if (group.items.length > 1) {
      const badgeRadius = 9;
      const badgeX = group.point.x + radius * 0.72;
      const badgeY = group.point.y - radius * 0.72;

      ctx.beginPath();
      ctx.fillStyle = '#ef4444';
      ctx.arc(badgeX, badgeY, badgeRadius, 0, Math.PI * 2);
      ctx.fill();

      ctx.strokeStyle = 'white';
      ctx.lineWidth = 1.5;
      ctx.stroke();

      ctx.fillStyle = 'white';
      ctx.font = 'bold 10px ui-sans-serif, system-ui, -apple-system, Segoe UI';
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      ctx.fillText(String(group.items.length), badgeX, badgeY + 0.5);
    }
  }
}

function renderSoon(): void {
  if (animationFrame !== null) {
    window.cancelAnimationFrame(animationFrame);
  }
  animationFrame = window.requestAnimationFrame(() => {
    drawScene();
    animationFrame = null;
  });
}

function getCanvasPoint(event: MouseEvent): { x: number; y: number } | null {
  const canvas = canvasRef.value;
  if (!canvas) return null;

  const rect = canvas.getBoundingClientRect();
  const x = event.clientX - rect.left;
  const y = event.clientY - rect.top;
  return { x, y };
}

function findClickedGroup(x: number, y: number): GraphGroup | null {
  for (let index = graphGroups.value.length - 1; index >= 0; index -= 1) {
    const group = graphGroups.value[index];
    if (!group) continue;

    const hitRadius = Math.max(group.radius, 10) + 6;
    const dx = x - group.point.x;
    const dy = y - group.point.y;
    if (Math.hypot(dx, dy) <= hitRadius) {
      return group;
    }
  }
  return null;
}

function openClusterMenu(group: GraphGroup): void {
  if (!containerRef.value) return;

  const menuWidth = 280;
  const menuHeight = Math.min(220, 52 + group.items.length * 32);

  const left = clamp(
    group.point.x + 18,
    12,
    Math.max(12, layout.value.width - menuWidth - 12),
  );

  const top = clamp(
    group.point.y - 16,
    12,
    Math.max(12, layout.value.height - menuHeight - 12),
  );

  clusterMenu.value = {
    visible: true,
    left,
    top,
    items: group.items,
  };
}

function closeClusterMenu(): void {
  clusterMenu.value.visible = false;
  clusterMenu.value.items = [];
}

function handleCanvasClick(event: MouseEvent): void {
  const point = getCanvasPoint(event);
  if (!point) return;

  const clickedGroup = findClickedGroup(point.x, point.y);
  if (!clickedGroup) {
    closeClusterMenu();
    return;
  }

  if (clickedGroup.items.length === 1) {
    const selected = clickedGroup.items[0];
    if (selected) {
      emit('select-item', selected);
    }
    closeClusterMenu();
    return;
  }

  openClusterMenu(clickedGroup);
}

function handleClusterItemSelect(item: Item): void {
  emit('select-item', item);
  closeClusterMenu();
}

watch(
  () => props.items,
  (items) => {
    updateData(items);
  },
  { deep: true, immediate: true },
);

watch(
  selectedRangeModel,
  (value) => {
    if (value && value !== selectedRange.value) selectedRange.value = value;
  },
  { immediate: true },
);

watch(selectedRange, (value) => {
  if (value !== selectedRangeModel.value) selectedRangeModel.value = value;
});

watch(
  selectedYFieldModel,
  (value) => {
    if (value && value !== selectedYField.value) selectedYField.value = value;
  },
  { immediate: true },
);

watch(selectedYField, (value) => {
  if (value !== selectedYFieldModel.value) selectedYFieldModel.value = value;
});

watch(
  selectedColorFieldModel,
  (value) => {
    if (value && value !== selectedColorField.value) selectedColorField.value = value;
  },
  { immediate: true },
);

watch(selectedColorField, (value) => {
  if (value !== selectedColorFieldModel.value) selectedColorFieldModel.value = value;
});

watch(
  selectedRadiusFieldModel,
  (value) => {
    if (value && value !== selectedRadiusField.value) selectedRadiusField.value = value;
  },
  { immediate: true },
);

watch(selectedRadiusField, (value) => {
  if (value !== selectedRadiusFieldModel.value) selectedRadiusFieldModel.value = value;
});

watch([selectedRange, selectedYField, selectedColorField, selectedRadiusField], () => {
  closeClusterMenu();
  refreshNow();
});

watch(
  [graphGroups, layout, warnings],
  async () => {
    closeClusterMenu();
    await nextTick();
    renderSoon();
  },
  { deep: true },
);

onMounted(async () => {
  await nextTick();
  updateViewportSize();
  await nextTick();
  updateViewportSize();

  resizeObserver = new ResizeObserver(() => {
    updateViewportSize();
    renderSoon();
  });

  if (containerRef.value) {
    resizeObserver.observe(containerRef.value);
  }

  refreshTimer = window.setInterval(() => {
    refreshNow();
  }, 5 * 60 * 1000);

  renderSoon();
});

onUnmounted(() => {
  resizeObserver?.disconnect();
  if (refreshTimer !== null) {
    window.clearInterval(refreshTimer);
  }
  if (animationFrame !== null) {
    window.cancelAnimationFrame(animationFrame);
  }
  if (sizeRetryFrame !== null) {
    window.cancelAnimationFrame(sizeRetryFrame);
  }
  destroy();
});
</script>
<template>
  <section class="canvas-wrapper">
    <div v-if="showDebug" class="debug-panel">
      <span class="label">Input</span> <strong>{{ debugStats.input }}</strong>
      <span class="dot">·</span>
      <span class="label">Visible</span> <strong>{{ debugStats.visible }}</strong>
      <span class="dot">·</span>
      <span class="label">Plotted</span> <strong>{{ debugStats.plotted }}</strong>
      <span class="dot">·</span>
      <span class="label">Skipped</span> 
      <strong :class="{ 'text-danger': debugStats.skipped > 0 }">{{ debugStats.skipped }}</strong>
    </div>

    <div ref="containerRef" class="canvas-viewport">
      <canvas
        ref="canvasRef"
        class="main-canvas"
        :style="stageStyle"
        @click="handleCanvasClick"
      />

      <div
        v-if="clusterMenu.visible"
        class="cluster-popup"
        :style="{ left: `${clusterMenu.left}px`, top: `${clusterMenu.top}px` }"
      >
        <p class="popup-header">Grouped Tasks</p>
        <ul class="popup-list">
          <li v-for="item in clusterMenu.items" :key="item.id">
            <button
              type="button"
              class="popup-item"
              @click="handleClusterItemSelect(item)"
            >
              {{ item.title }}
            </button>
          </li>
        </ul>
      </div>

      <div
        v-if="warnings.length > 0"
        class="canvas-warning"
      >
        <p v-for="warning in warnings" :key="warning" class="warning-text">
          <span class="warning-icon">⚠️</span> {{ warning }}
        </p>
      </div>
    </div>
  </section>
</template>

<template>
  <section class="canvas-wrapper">
    <div v-if="showDebug" class="debug-panel">
      <span class="label">Input</span> <strong>{{ debugStats.input }}</strong>
      <span class="dot">·</span>
      <span class="label">Visible</span> <strong>{{ debugStats.visible }}</strong>
      <span class="dot">·</span>
      <span class="label">Plotted</span> <strong>{{ debugStats.plotted }}</strong>
      <span class="dot">·</span>
      <span class="label">Skipped</span> 
      <strong :class="{ 'text-danger': debugStats.skipped > 0 }">{{ debugStats.skipped }}</strong>
    </div>

    <div ref="containerRef" class="canvas-viewport">
      <canvas
        ref="canvasRef"
        class="main-canvas"
        :style="stageStyle"
        @click="handleCanvasClick"
      />

      <div
        v-if="clusterMenu.visible"
        class="cluster-popup"
        :style="{ left: `${clusterMenu.left}px`, top: `${clusterMenu.top}px` }"
      >
        <p class="popup-header">Grouped Tasks</p>
        <ul class="popup-list">
          <li v-for="item in clusterMenu.items" :key="item.id">
            <button
              type="button"
              class="popup-item"
              @click="handleClusterItemSelect(item)"
            >
              {{ item.title }}
            </button>
          </li>
        </ul>
      </div>

      <div
        v-if="warnings.length > 0"
        class="canvas-warning"
      >
        <p v-for="warning in warnings" :key="warning" class="warning-text">
          <span class="warning-icon">⚠️</span> {{ warning }}
        </p>
      </div>
    </div>
  </section>
</template>

<style scoped>
/* --- Container --- */
.canvas-wrapper {
  position: relative;
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  gap: 0.75rem;
}

/* --- Debug Panel --- */
.debug-panel {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  background: rgba(255, 255, 255, 0.7);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(226, 232, 240, 0.8);
  border-radius: 10px;
  font-size: 0.7rem;
  color: #64748b;
  width: fit-content;
}

.debug-panel strong {
  color: #1e293b;
  font-weight: 800;
}

.debug-panel .text-danger {
  color: #ef4444;
}

.debug-panel .label {
  color: #94a3b8;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.debug-panel .dot {
  color: #e2e8f0;
}

/* --- Main Canvas Viewport --- */
.canvas-viewport {
  position: relative;
  flex: 1;
  min-height: 0;
  overflow: hidden;
  background: white;
  border-radius: 20px;
  border: 1px solid rgba(226, 232, 240, 0.8);
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.03);
}

.main-canvas {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  cursor: crosshair; /* ツール感を出すために十字カーソルに */
}

/* --- Cluster Popup (Glassmorphism) --- */
.cluster-popup {
  position: absolute;
  z-index: 30;
  width: 260px;
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.8);
  border-radius: 16px;
  padding: 0.75rem;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
  animation: popup-in 0.2s ease-out;
}

.popup-header {
  font-size: 0.65rem;
  font-weight: 800;
  color: #94a3b8;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  margin-bottom: 0.75rem;
  padding-left: 0.5rem;
}

.popup-list {
  max-height: 200px;
  overflow-y: auto;
  margin: 0;
  padding: 0;
  list-style: none;
}

/* スクロールバーのカスタマイズ */
.popup-list::-webkit-scrollbar {
  width: 4px;
}
.popup-list::-webkit-scrollbar-thumb {
  background: #e2e8f0;
  border-radius: 10px;
}

.popup-item {
  width: 100%;
  padding: 0.6rem 0.75rem;
  text-align: left;
  font-size: 0.85rem;
  font-weight: 600;
  color: #475569;
  background: transparent;
  border: none;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s;
}

.popup-item:hover {
  background: rgba(168, 85, 247, 0.08);
  color: #a855f7;
}

/* --- Warnings --- */
.canvas-warning {
  pointer-events: none;
  position: absolute;
  bottom: 1.25rem;
  left: 50%;
  transform: translateX(-50%);
  z-index: 20;
  width: min(90%, 600px);
  background: rgba(255, 251, 235, 0.9);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(251, 191, 36, 0.3);
  border-radius: 12px;
  padding: 0.6rem 1rem;
  box-shadow: 0 10px 20px rgba(180, 83, 9, 0.05);
}

.warning-text {
  font-size: 0.75rem;
  color: #b45309;
  margin: 0;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.warning-icon {
  font-size: 0.85rem;
}

/* --- Animations --- */
@keyframes popup-in {
  from { opacity: 0; transform: scale(0.95) translateY(10px); }
  to { opacity: 1; transform: scale(1) translateY(0); }
}
</style>