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
  <section class="relative flex h-full min-h-0 flex-col gap-2">
    <div v-if="showDebug" class="rounded-lg border border-slate-200 bg-white/70 px-3 py-2 text-xs text-slate-700">
      Input <strong>{{ debugStats.input }}</strong>
      · Visible <strong>{{ debugStats.visible }}</strong>
      · Plotted <strong>{{ debugStats.plotted }}</strong>
      · Skipped <strong :class="debugStats.skipped > 0 ? 'text-red-600' : ''">{{ debugStats.skipped }}</strong>
    </div>

    <div ref="containerRef" class="relative min-h-0 flex-1 overflow-hidden rounded-2xl border border-slate-200 bg-white">
      <canvas
        ref="canvasRef"
        class="absolute inset-0 h-full w-full cursor-pointer"
        :style="stageStyle"
        @click="handleCanvasClick"
      />

      <div
        v-if="clusterMenu.visible"
        class="absolute z-30 w-[280px] rounded-xl border border-slate-200 bg-white/95 p-3 shadow-xl backdrop-blur"
        :style="{ left: `${clusterMenu.left}px`, top: `${clusterMenu.top}px` }"
      >
        <p class="mb-2 text-xs font-semibold uppercase tracking-wide text-slate-500">Grouped tasks</p>
        <ul class="max-h-48 space-y-1 overflow-auto">
          <li v-for="item in clusterMenu.items" :key="item.id">
            <button
              type="button"
              class="w-full rounded-lg px-2 py-1.5 text-left text-sm text-slate-700 hover:bg-slate-100"
              @click="handleClusterItemSelect(item)"
            >
              {{ item.title }}
            </button>
          </li>
        </ul>
      </div>

      <div
        v-if="warnings.length > 0"
        class="pointer-events-none absolute bottom-3 left-1/2 z-20 w-[min(92%,720px)] -translate-x-1/2 rounded-xl border border-amber-300 bg-amber-50/90 px-3 py-2 text-xs text-amber-900 shadow-sm backdrop-blur"
      >
        <p v-for="warning in warnings" :key="warning">{{ warning }}</p>
      </div>
    </div>
  </section>
</template>
