import { ref, type Ref } from 'vue';
import type { Item } from '@/types/item';
import type { GraphItem, GraphConfig } from '@/types/graph';

export function useGraph(
  canvasRef: Ref<HTMLCanvasElement | null>,
  config: GraphConfig
) {
  // hold GraphItem data as a Ref to trigger reactivity when updated
  const graphItems = ref<GraphItem[]>([]);

  /**
   * load raw items and calculate their positions/colors for graph rendering
   */
  const updateData = (rawItems: Item[]) => {
    const canvas = canvasRef.value;
    if (!canvas) return;

    const activeItems = rawItems.filter(i => i.status !== 'done' && !i.deleted_at);
    if (activeItems.length === 0) {
      graphItems.value = [];
      draw();
      return;
    }

    // calculate time range for x-axis scaling
    const times = activeItems.map(i => new Date(i.due).getTime());
    const minTime = Math.min(...times);
    const maxTime = Math.max(...times);
    const timeRange = (maxTime - minTime) || 86400000;

    // calculate graph dimensions accounting for padding
    const width = (canvas.width / (window.devicePixelRatio || 1)) - config.padding.left - config.padding.right;
    const height = (canvas.height / (window.devicePixelRatio || 1)) - config.padding.top - config.padding.bottom;

    // map items to GraphItem with calculated positions and colors
    graphItems.value = activeItems.map(item => {
      const t = new Date(item.due).getTime();
      const x = config.padding.left + ((t - minTime) / timeRange) * width;
      const m = item.motivation ?? config.defaultMotivation;
      const y = (canvas.height / (window.devicePixelRatio || 1)) - config.padding.bottom - (m / 100) * height;

      return {
        ...item,
        point: { x, y },
        radius: 6,
        color: item.status === 'inprogress' ? '#3b82f6' : '#94a3b8'
      };
    });

    draw();
  };

  /**
   * draw the graph on the canvas based on current graphItems and config
   */
  const draw = () => {
    const canvas = canvasRef.value;
    const ctx = canvas?.getContext('2d');
    if (!canvas || !ctx) return;

    const dpr = window.devicePixelRatio || 1;
    const logicalWidth = canvas.width / dpr;
    const logicalHeight = canvas.height / dpr;

    ctx.clearRect(0, 0, logicalWidth, logicalHeight);
    
    drawGrid(ctx, logicalWidth, logicalHeight);

    graphItems.value.forEach(item => {
      ctx.beginPath();
      ctx.arc(item.point.x, item.point.y, item.radius, 0, Math.PI * 2);
      ctx.fillStyle = item.color;
      ctx.fill();
      ctx.strokeStyle = '#fff';
      ctx.lineWidth = 2;
      ctx.stroke();
    });
  };

  // hold drawGrid as a separate function for clarity
  const drawGrid = (ctx: CanvasRenderingContext2D, w: number, h: number) => {
    ctx.strokeStyle = '#e2e8f0';
    ctx.lineWidth = 1;
    ctx.font = '10px sans-serif';
    ctx.fillStyle = '#94a3b8';

    config.gridLines.forEach(m => {
      const height = h - config.padding.top - config.padding.bottom;
      const y = h - config.padding.bottom - (m / 100) * height;
      ctx.beginPath();
      ctx.moveTo(config.padding.left, y);
      ctx.lineTo(w - config.padding.right, y);
      ctx.stroke();
      ctx.fillText(`${m}%`, 10, y + 3);
    });
  };

  return {
    graphItems,
    updateData,
    draw
  };
}