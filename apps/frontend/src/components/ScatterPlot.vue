<script setup lang="ts">
import { ref, onMounted, watch, onUnmounted } from 'vue';
import type { Item } from '@/types/item';
import type { GraphItem, GraphConfig } from '@/types/graph';
import { useGraph } from '@/composables/useGraph';

const props = defineProps<{
  items: Item[]
}>();

const graphConfig: GraphConfig = {
  padding: { top: 40, right: 40, bottom: 60, left: 60 },
  gridLines: [0, 25, 50, 75, 100],
  defaultMotivation: 50
};

const containerRef = ref<HTMLDivElement | null>(null);
const canvasRef = ref<HTMLCanvasElement | null>(null);
const hoveredItem = ref<GraphItem | null>(null);

// initialize useGraph composable with canvas ref and items
const { graphItems, updateData, draw } = useGraph(canvasRef, graphConfig);

watch(() => props.items, (newItems) => {
  updateData(newItems);
}, { deep: true, immediate: true });

// detect change of dimention with ResizeObserver
let resizeObserver: ResizeObserver | null = null;

const updateCanvasSize = () => {
  const canvas = canvasRef.value;
  const container = containerRef.value;
  if (!canvas || !container) return;

  const { width, height } = container.getBoundingClientRect();
  const dpr = window.devicePixelRatio || 1;

  // set CSS size (logical pixels)
  canvas.style.width = `${width}px`;
  canvas.style.height = `${height}px`;

  // set actual size (physical pixels) for high-DPI rendering
  canvas.width = width * dpr;
  canvas.height = height * dpr;

  // scale the drawing context to account for the device pixel ratio
  const ctx = canvas.getContext('2d');
  if (ctx) ctx.scale(dpr, dpr);

  updateData(props.items);
};

const handleMouseMove = (event: MouseEvent) => {
  const rect = canvasRef.value?.getBoundingClientRect();
  if (!rect) return;

  const x = event.clientX - rect.left;
  const y = event.clientY - rect.top;

  // look for hovered item (within radius + some margin)
  hoveredItem.value = graphItems.value.find(item => {
    const dx = item.point.x - x;
    const dy = item.point.y - y;
    return Math.sqrt(dx * dx + dy * dy) < item.radius + 4; // 4px margin for easier hovering
  }) || null;
};

const handleMouseLeave = () => {
  hoveredItem.value = null;
};

onMounted(() => {
  updateCanvasSize();
  resizeObserver = new ResizeObserver(updateCanvasSize);
  if (containerRef.value) resizeObserver.observe(containerRef.value);
});

onUnmounted(() => {
  resizeObserver?.disconnect();
});

watch(() => props.items, (newItems) => {
  updateData(newItems);
   draw();
}, { deep: true, immediate: true });

</script>

<template>
  <div ref="containerRef" class="w-full h-full min-h-[400px] relative overflow-hidden bg-white rounded-xl shadow-inner">
    <canvas
      ref="canvasRef"
      @mousemove="handleMouseMove"
      @mouseleave="handleMouseLeave"
      class="block cursor-crosshair"
    ></canvas>
    
    <div v-if="hoveredItem" 
         :style="{ left: hoveredItem.point.x + 10 + 'px', top: hoveredItem.point.y - 40 + 'px' }"
         class="absolute z-10 p-2 bg-gray-800 text-white text-xs rounded pointer-events-none shadow-lg whitespace-nowrap">
      <strong>{{ hoveredItem.title }}</strong><br/>
      Motivation: {{ hoveredItem.motivation ?? 50 }}%
    </div>
  </div>
</template>