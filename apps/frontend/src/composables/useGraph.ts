import { ref, type Ref } from 'vue';
import {
  extent,
  forceCollide,
  forceSimulation,
  forceX,
  forceY,
  max,
  scaleLinear,
  scalePoint,
} from 'd3';
import type { Item } from '@/types/item';
import type {
  GraphAxisField,
  GraphConfig,
  GraphGroup,
  GraphItem,
  GraphLayout,
  GraphTick,
  GraphTimeRangeKey,
  GraphVisualField,
} from '@/types/graph';

const STATUS_VALUE: Record<Item['status'], number> = {
  backlog: 1,
  todo: 2,
  inprogress: 3,
  done: 4,
};

const STATUS_COLOR: Record<Item['status'], string> = {
  backlog: '#64748b',
  todo: '#94a3b8',
  inprogress: '#2563eb',
  done: '#16a34a',
};

const EMPTY_LAYOUT: GraphLayout = {
  width: 0,
  height: 0,
  plotLeft: 0,
  plotRight: 0,
  plotTop: 0,
  plotBottom: 0,
  innerWidth: 0,
  innerHeight: 0,
  originX: 0,
  yMax: 0,
  nowMs: 0,
};

/**
 * 100 Program brand gradient: Redish-Pink → Purple → Deep Blue
 * Maps normalized values (0-1) to a three-stop color gradient
 * @param t Normalized value between 0 and 1
 * @returns Hex color string
 */
function interpolate100ProgramGradient(t: number): string {
  // Ensure t is clamped between 0 and 1
  const normalized = Math.min(Math.max(t, 0), 1);

  // Three-color gradient stops
  type ColorStop = {
    pos: number;
    color: [number, number, number];
  };

  const colorStops: ColorStop[] = [
    { pos: 0.0, color: [0xe6, 0x39, 0x46] }, // Redish-Pink #E63946
    { pos: 0.5, color: [0xb4, 0x5f, 0xd1] }, // Purple #B45FD1
    { pos: 1.0, color: [0x25, 0x63, 0xeb] }, // Deep Blue #2563EB
  ];

  // Find which two stops we're between
  const firstStop = colorStops[0];
  const secondStop = colorStops[1];

  if (!firstStop || !secondStop) {
    return '#E63946'; // Fallback to first color
  }

  let lowStop: ColorStop = firstStop;
  let highStop: ColorStop = secondStop;

  for (let i = 0; i < colorStops.length - 1; i += 1) {
    const current = colorStops[i];
    const next = colorStops[i + 1];

    if (!current || !next) continue;

    if (normalized >= current.pos && normalized <= next.pos) {
      lowStop = current;
      highStop = next;
      break;
    }
  }

  // Interpolate between the two stops
  const range = highStop.pos - lowStop.pos;
  const ratio = range === 0 ? 0 : (normalized - lowStop.pos) / range;

  const r = Math.round(lowStop.color[0] + (highStop.color[0] - lowStop.color[0]) * ratio);
  const g = Math.round(lowStop.color[1] + (highStop.color[1] - lowStop.color[1]) * ratio);
  const b = Math.round(lowStop.color[2] + (highStop.color[2] - lowStop.color[2]) * ratio);

  return `#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}`.toUpperCase();
}

function clamp(value: number, minValue: number, maxValue: number): number {
  return Math.min(Math.max(value, minValue), maxValue);
}

function mean(values: number[]): number {
  if (values.length === 0) return 0;
  return values.reduce((total, value) => total + value, 0) / values.length;
}

function formatRelativeTime(ms: number): string {
  if (ms === 0) return 'Now';

  const sign = ms < 0 ? '-' : '+';
  const absolute = Math.abs(ms);
  const hour = 60 * 60 * 1000;
  const day = 24 * hour;

  if (absolute < day) {
    return `${sign}${Math.round(absolute / hour)}h`;
  }

  const days = absolute / day;
  return `${sign}${days >= 10 ? Math.round(days) : days.toFixed(1)}d`;
}

function formatNumericTick(value: number): string {
  if (value >= 100) return Math.round(value).toString();
  if (value >= 10) return value.toFixed(0);
  return value.toFixed(1).replace(/\.0$/, '');
}

function getMetricValue(
  item: Item,
  field: GraphAxisField | GraphVisualField,
  defaultMotivation: number,
): number | null {
  if (field === 'none') return null;

  if (field === 'status') {
    return STATUS_VALUE[item.status];
  }

  if (field === 'motivation') {
    return typeof item.motivation === 'number' ? item.motivation : defaultMotivation;
  }

  return typeof item.duration_minutes === 'number' ? item.duration_minutes : null;
}

function buildXTicks(xScale: (value: number) => number, rangeMs: number): GraphTick[] {
  const tickValues = [-rangeMs, -rangeMs / 2, 0, rangeMs / 2, rangeMs];
  return tickValues.map((value) => ({
    position: xScale(value),
    label: formatRelativeTime(value),
  }));
}

function buildYTicks(
  yScale: ((value: number) => number) | ((value: Item['status']) => number | undefined),
  yMax: number,
  field: GraphAxisField,
): GraphTick[] {
  if (field === 'status') {
    const statusOrder: Item['status'][] = ['backlog', 'todo', 'inprogress', 'done'];
    const statusLabels: Record<Item['status'], string> = {
      backlog: 'Backlog',
      todo: 'Todo',
      inprogress: 'In Progress',
      done: 'Done',
    };

    return statusOrder.map((status) => ({
      position: (yScale as (value: Item['status']) => number | undefined)(status) ?? 0,
      label: statusLabels[status],
    }));
  }

  return [0, 0.25, 0.5, 0.75, 1].map((ratio) => {
    const value = yMax * ratio;
    return {
      position: (yScale as (value: number) => number)(value),
      label: formatNumericTick(value),
    };
  });
}

function parseDueMs(item: Item): number | null {
  const ms = Date.parse(item.due);
  if (!Number.isFinite(ms)) {
    return null;
  }
  return ms;
}

export type GraphDebugStats = {
  input: number;
  visible: number;
  plotted: number;
  skipped: number;
};

type GraphControls = {
  selectedRange?: Ref<GraphTimeRangeKey>;
  selectedYField?: Ref<GraphAxisField>;
  selectedColorField?: Ref<GraphVisualField>;
  selectedRadiusField?: Ref<GraphVisualField>;
  groupingEnabled?: Ref<boolean>;
};

export function useGraph(config: GraphConfig, controls: GraphControls = {}) {
  const graphItems = ref<GraphItem[]>([]);
  const graphGroups = ref<GraphGroup[]>([]);
  const warnings = ref<string[]>([]);
  const layout = ref<GraphLayout>(EMPTY_LAYOUT);
  const xTicks = ref<GraphTick[]>([]);
  const yTicks = ref<GraphTick[]>([]);
  const debugStats = ref<GraphDebugStats>({
    input: 0,
    visible: 0,
    plotted: 0,
    skipped: 0,
  });

  const selectedRange = controls.selectedRange ?? ref<GraphTimeRangeKey>('1w');
  const selectedYField = controls.selectedYField ?? ref<GraphAxisField>('duration_minutes');
  const selectedColorField = controls.selectedColorField ?? ref<GraphVisualField>('motivation');
  const selectedRadiusField = controls.selectedRadiusField ?? ref<GraphVisualField>('duration_minutes');
  const groupingEnabled = controls.groupingEnabled ?? ref(false);

  let rawItems: Item[] = [];
  let simulation: ReturnType<typeof forceSimulation<GraphItem>> | null = null;

  function aggregateByDistance(items: GraphItem[]): GraphGroup[] {
    if (!groupingEnabled.value || items.length <= 1) {
      return items.map((item) => ({
        key: `${item.id}:single`,
        items: [item],
        point: item.point,
        radius: item.radius,
        color: item.color,
        marker: item.marker,
        label: '',
        representative: item,
      }));
    }

    const threshold = config.groupDistance;
    const visited = new Set<number>();
    const groups: GraphGroup[] = [];

    for (let i = 0; i < items.length; i += 1) {
      if (visited.has(i)) continue;

      const queue: number[] = [i];
      const clusterIndexes: number[] = [];
      visited.add(i);

      while (queue.length > 0) {
        const currentIndex = queue.shift();
        if (currentIndex === undefined) continue;
        clusterIndexes.push(currentIndex);

        for (let j = 0; j < items.length; j += 1) {
          if (visited.has(j)) continue;

          const a = items[currentIndex];
          const b = items[j];
          if (!a || !b) continue;

          const dx = a.point.x - b.point.x;
          const dy = a.point.y - b.point.y;
          const distance = Math.hypot(dx, dy);

          if (distance <= threshold) {
            visited.add(j);
            queue.push(j);
          }
        }
      }

      const clusterItems = clusterIndexes.map((index) => items[index]).filter((item): item is GraphItem => Boolean(item));
      if (clusterItems.length === 0) continue;

      const representative = [...clusterItems].sort((left, right) => {
        const leftPriority = Math.abs(left.timeDiffMs);
        const rightPriority = Math.abs(right.timeDiffMs);
        return leftPriority - rightPriority;
      })[0] ?? clusterItems[0];

      if (!representative) continue;

      const point = {
        x: mean(clusterItems.map((item) => item.point.x)),
        y: mean(clusterItems.map((item) => item.point.y)),
      };

      const radius = Math.max(
        representative.radius,
        config.minRadius + Math.max(0, Math.sqrt(clusterItems.length - 1) * 3),
      );

      groups.push({
        key: `${representative.id}:cluster:${clusterItems.length}:${Math.round(point.x)}:${Math.round(point.y)}`,
        items: clusterItems,
        point,
        radius,
        color: representative.color,
        marker: representative.marker,
        label: clusterItems.length > 1 ? String(clusterItems.length) : '',
        representative,
      });
    }

    return groups;
  }

  function recompute(): void {
    if (layout.value.width <= 0 || layout.value.height <= 0) return;

    const nowMs = Date.now();
    const rangeMs = config.timeRanges[selectedRange.value];
    const plotLeft = config.padding.left;
    const plotRight = layout.value.width - config.padding.right;
    const plotTop = config.padding.top;
    const plotBottom = layout.value.height - config.padding.bottom;
    const innerWidth = Math.max(plotRight - plotLeft, 1);
    const innerHeight = Math.max(plotBottom - plotTop, 1);

    const visibleItems = rawItems.filter((item) => !item.deleted_at && !item.is_archived);
    const parsedItems = visibleItems
      .map((item) => {
        const dueMs = parseDueMs(item);
        return dueMs === null ? null : { item, dueMs };
      })
      .filter((entry): entry is { item: Item; dueMs: number } => entry !== null);

    const skipped = visibleItems.length - parsedItems.length;
    debugStats.value = {
      input: rawItems.length,
      visible: visibleItems.length,
      plotted: parsedItems.length,
      skipped,
    };

    const xScale = scaleLinear().domain([-rangeMs, rangeMs]).range([plotLeft, plotRight]);

    if (parsedItems.length === 0) {
      simulation?.stop();
      simulation = null;
      graphItems.value = [];
      graphGroups.value = [];
      warnings.value = skipped > 0 ? [`Skipped ${skipped} item(s) due to invalid due timestamps.`] : [];
      layout.value = {
        width: layout.value.width,
        height: layout.value.height,
        plotLeft,
        plotRight,
        plotTop,
        plotBottom,
        innerWidth,
        innerHeight,
        originX: xScale(0),
        yMax: 0,
        nowMs,
      };
      xTicks.value = buildXTicks(xScale, rangeMs);
      yTicks.value = [];
      return;
    }

    const yWarnings: string[] = [];
    const yField = selectedYField.value;
    const colorField = selectedColorField.value;
    const radiusField = selectedRadiusField.value;

    const missingYCount = parsedItems.filter(({ item }) => getMetricValue(item, yField, config.defaultMotivation) === null).length;
    if (missingYCount > 0) {
      yWarnings.push(`Field ${yField} is missing on ${missingYCount} item(s).`);
    }

    let yScaleNumeric = scaleLinear().domain([0, 1]).range([plotBottom, plotTop]);
    let yScaleStatus = scalePoint<Item['status']>()
      .domain(['backlog', 'todo', 'inprogress', 'done'])
      .range([plotBottom, plotTop])
      .padding(0.25);

    let yMax = 4;
    if (yField !== 'status') {
      const yValues = parsedItems.map(({ item }) => getMetricValue(item, yField, config.defaultMotivation) ?? 0);
      yMax = Math.max(max(yValues) ?? 0, 1);
      yScaleNumeric = scaleLinear().domain([0, yMax]).range([plotBottom, plotTop]);
    }

    const colorValues = parsedItems
      .map(({ item }) => getMetricValue(item, colorField, config.defaultMotivation))
      .filter((value): value is number => value !== null);

    const colorExtent = extent(colorValues);
    const colorMin = colorExtent[0] ?? 0;
    const colorMax = colorExtent[1] ?? 1;

    const radiusValues = parsedItems
      .map(({ item }) => getMetricValue(item, radiusField, config.defaultMotivation))
      .filter((value): value is number => value !== null);

    const radiusExtent = extent(radiusValues);
    const radiusMin = radiusExtent[0] ?? 0;
    const radiusMax = radiusExtent[1] ?? radiusMin + 1;
    const radiusScale = scaleLinear()
      .domain(radiusMin === radiusMax ? [radiusMin, radiusMin + 1] : [radiusMin, radiusMax])
      .range([config.minRadius, config.maxRadius]);

    const previousItemsById = new Map(graphItems.value.map((item) => [item.id, item]));

    const nextItems: GraphItem[] = parsedItems.map(({ item, dueMs }) => {
      const timeDiffMs = dueMs - nowMs;
      const clampedDiff = clamp(timeDiffMs, -rangeMs, rangeMs);

      // X axis zones: past (<0), now (=0), future (>0)
      const marker =
        timeDiffMs < -rangeMs
          ? 'triangle-left'
          : timeDiffMs > rangeMs
            ? 'triangle-right'
            : 'circle';

      const yMetric = getMetricValue(item, yField, config.defaultMotivation);
      const yValue = yMetric ?? 0;

      const targetX = xScale(clampedDiff);
      const targetY =
        yField === 'status'
          ? yScaleStatus(item.status) ?? plotBottom
          : yScaleNumeric(yValue);

      const colorMetric = getMetricValue(item, colorField, config.defaultMotivation);
      let color = config.neutralColor;
      if (colorField === 'status') {
        color = STATUS_COLOR[item.status];
      } else if (colorField === 'motivation') {
        const motivation = typeof item.motivation === 'number' ? item.motivation : config.defaultMotivation;
        const normalized = clamp((motivation - 1) / 9, 0, 1);
        color = interpolate100ProgramGradient(normalized);
      } else if (colorField !== 'none' && colorMetric !== null) {
        const normalized = colorMax === colorMin ? 0.5 : (colorMetric - colorMin) / (colorMax - colorMin);
        color = interpolate100ProgramGradient(clamp(normalized, 0, 1));
      }

      const radiusMetric = getMetricValue(item, radiusField, config.defaultMotivation);
      let radius = config.defaultRadius;
      if (radiusField === 'status') {
        radius = radiusScale(STATUS_VALUE[item.status]);
      } else if (radiusField !== 'none' && radiusMetric !== null) {
        radius = radiusScale(radiusMetric);
      }

      const previous = previousItemsById.get(item.id);
      const safeTarget = {
        x: Number.isFinite(targetX) ? targetX : xScale(0),
        y: Number.isFinite(targetY) ? targetY : plotBottom,
      };

      return {
        ...item,
        point: safeTarget,
        target: safeTarget,
        radius: Number.isFinite(radius) && radius > 0 ? radius : config.defaultRadius,
        color,
        marker,
        timeDiffMs,
        yValue,
        isOverdue: timeDiffMs < 0,
        isClamped: marker !== 'circle',
        priority: Math.abs(timeDiffMs),
        x: previous?.x ?? safeTarget.x,
        y: previous?.y ?? safeTarget.y,
        vx: previous?.vx ?? 0,
        vy: previous?.vy ?? 0,
      };
    });

    simulation?.stop();
    simulation = forceSimulation(nextItems)
      .force('x', forceX<GraphItem>((item) => item.target.x).strength(0.9))
      .force('y', forceY<GraphItem>((item) => item.target.y).strength(0.9))
      .force(
        'collide',
        forceCollide<GraphItem>((item) => item.radius + config.collisionPadding).iterations(2),
      )
      .stop();

    const ticks = Math.max(90, Math.min(120, config.tickCount));
    for (let tick = 0; tick < ticks; tick += 1) {
      simulation.tick();
    }

    nextItems.forEach((item) => {
      item.point = {
        x: clamp(item.x ?? item.target.x, plotLeft, plotRight),
        y: clamp(item.y ?? item.target.y, plotTop, plotBottom),
      };
    });

    graphItems.value = nextItems;
    graphGroups.value = aggregateByDistance(nextItems);
    warnings.value = [
      ...yWarnings,
      ...(skipped > 0 ? [`Skipped ${skipped} item(s) due to invalid due timestamps.`] : []),
    ];

    layout.value = {
      width: layout.value.width,
      height: layout.value.height,
      plotLeft,
      plotRight,
      plotTop,
      plotBottom,
      innerWidth,
      innerHeight,
      originX: xScale(0),
      yMax,
      nowMs,
    };

    xTicks.value = buildXTicks(xScale, rangeMs);
    yTicks.value =
      yField === 'status'
        ? buildYTicks(yScaleStatus as (value: Item['status']) => number | undefined, yMax, yField)
        : buildYTicks(yScaleNumeric as (value: number) => number, yMax, yField);
  }

  function setDimensions(width: number, height: number): void {
    const safeWidth = Math.max(Math.round(width), 1);
    const safeHeight = Math.max(Math.round(height), 1);

    if (safeWidth === layout.value.width && safeHeight === layout.value.height) {
      return;
    }

    layout.value = {
      ...layout.value,
      width: safeWidth,
      height: safeHeight,
    };

    recompute();
  }

  function updateData(items: Item[]): void {
    rawItems = items;
    recompute();
  }

  function refreshNow(): void {
    recompute();
  }

  function destroy(): void {
    simulation?.stop();
    simulation = null;
  }

  return {
    graphItems,
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
  };
}
