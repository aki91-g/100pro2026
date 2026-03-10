import { ref } from 'vue';
import {
  extent,
  forceCollide,
  forceSimulation,
  forceX,
  forceY,
  max,
  scaleLinear,
  scaleQuantize,
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

const STATUS_PRIORITY: Record<Item['status'], number> = {
  backlog: 2,
  todo: 3,
  inprogress: 4,
  done: 1,
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

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.min(Math.max(value, minimum), maximum);
}

function mean(values: number[]): number {
  if (values.length === 0) return 0;
  return values.reduce((sum, value) => sum + value, 0) / values.length;
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
  if (field === 'status') return STATUS_VALUE[item.status];
  if (field === 'motivation') {
    return typeof item.motivation === 'number' ? item.motivation : defaultMotivation;
  }
  return typeof item.duration_minutes === 'number' ? item.duration_minutes : null;
}

function getPriorityScore(item: Item, timeDiffMs: number): number {
  const overdueBoost = timeDiffMs < 0 ? 100 : 0;
  const motivationBoost = typeof item.motivation === 'number' ? item.motivation : 0;
  return overdueBoost + STATUS_PRIORITY[item.status] * 10 + motivationBoost;
}

export type GraphDebugStats = {
  inputItems: number;
  visibleItems: number;
  plottedItems: number;
  invalidDueCount: number;
};

export function useGraph(config: GraphConfig) {
  const graphItems = ref<GraphItem[]>([]);
  const graphGroups = ref<GraphGroup[]>([]);
  const warnings = ref<string[]>([]);
  const layout = ref<GraphLayout>(EMPTY_LAYOUT);
  const xTicks = ref<GraphTick[]>([]);
  const yTicks = ref<GraphTick[]>([]);
  const debugStats = ref<GraphDebugStats>({
    inputItems: 0,
    visibleItems: 0,
    plottedItems: 0,
    invalidDueCount: 0,
  });

  const selectedRange = ref<GraphTimeRangeKey>('1w');
  const selectedYField = ref<GraphAxisField>('duration_minutes');
  const selectedColorField = ref<GraphVisualField>('motivation');
  const selectedRadiusField = ref<GraphVisualField>('duration_minutes');
  const groupingEnabled = ref(false);

  let rawItems: Item[] = [];
  let simulation: ReturnType<typeof forceSimulation<GraphItem>> | null = null;

  const buildXTicks = (xScale: (value: number) => number, rangeMs: number): GraphTick[] => {
    const tickValues = [-rangeMs, -rangeMs / 2, 0, rangeMs / 2, rangeMs];
    return tickValues.map((value) => ({
      position: xScale(value),
      label: formatRelativeTime(value),
    }));
  };

  const buildYTicks = (
    yScale: (value: number) => number,
    yMax: number,
    field: GraphAxisField,
  ): GraphTick[] => {
    if (field === 'status') {
      return [
        { position: yScale(1), label: 'Backlog' },
        { position: yScale(2), label: 'Todo' },
        { position: yScale(3), label: 'In Progress' },
        { position: yScale(4), label: 'Done' },
      ];
    }

    return [0, 0.25, 0.5, 0.75, 1].map((ratio) => {
      const value = yMax * ratio;
      return {
        position: yScale(value),
        label: formatNumericTick(value),
      };
    });
  };

  const aggregateGroups = (items: GraphItem[]): GraphGroup[] => {
    const groups = new Map<string, GraphItem[]>();

    items.forEach((item) => {
      const bucketX = Math.round(item.point.x / config.groupDistance);
      const bucketY = Math.round(item.point.y / config.groupDistance);
      const key = `${bucketX}:${bucketY}:${item.marker}`;
      const existing = groups.get(key);
      if (existing) {
        existing.push(item);
      } else {
        groups.set(key, [item]);
      }
    });

    return Array.from(groups.entries()).flatMap(([bucketKey, bucketItems]) => {
      if (bucketItems.length === 0) {
        return [];
      }

      const sorted = [...bucketItems].sort((left, right) => right.priority - left.priority);
      const representative: GraphItem = sorted[0] ?? bucketItems[0]!;
      const point = {
        x: mean(bucketItems.map((item) => item.point.x)),
        y: mean(bucketItems.map((item) => item.point.y)),
      };
      const radius = Math.max(
        representative.radius,
        config.minRadius + Math.max(0, Math.sqrt(bucketItems.length - 1) * 3),
      );

      return [{
        key: `${representative.id}:${bucketKey}:${bucketItems.length}`,
        items: bucketItems,
        point,
        radius,
        color: representative.color,
        marker: representative.marker,
        label: bucketItems.length > 1 ? String(bucketItems.length) : '',
        representative,
      }];
    });
  };

  const singleItemGroups = (items: GraphItem[]): GraphGroup[] => {
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
  };

  const parseDueMs = (item: Item): number | null => {
    const ms = Date.parse(item.due);
    if (!Number.isFinite(ms)) {
      console.error('[useGraph] Invalid due timestamp:', { id: item.id, title: item.title, due: item.due });
      return null;
    }
    return ms;
  };

  const recompute = (): void => {
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

    const xScale = scaleLinear().domain([-rangeMs, rangeMs]).range([plotLeft, plotRight]);

    if (visibleItems.length === 0 || parsedItems.length === 0) {
      simulation?.stop();
      graphItems.value = [];
      graphGroups.value = [];
      const invalidDueCount = visibleItems.length - parsedItems.length;
      warnings.value =
        invalidDueCount > 0
          ? [`No plottable items: ${invalidDueCount} item(s) have invalid due timestamps.`]
          : [];
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

    const missingYItems = parsedItems
      .map((entry) => entry.item)
      .filter(
      (item) => getMetricValue(item, selectedYField.value, config.defaultMotivation) === null,
    );

    const yValues = parsedItems.map((entry) => {
      const item = entry.item;
      const value = getMetricValue(item, selectedYField.value, config.defaultMotivation);
      return value ?? 0;
    });

    const yMax = Math.max(max(yValues) ?? 0, 1);
    const yScale = scaleLinear().domain([0, yMax]).range([plotBottom, plotTop]);

    let colorScale: ((value: number) => string) | null = null;
    if (selectedColorField.value !== 'none' && selectedColorField.value !== 'status') {
      const colorValues = parsedItems
        .map((entry) => getMetricValue(entry.item, selectedColorField.value, config.defaultMotivation))
        .filter((value): value is number => value !== null);
      const colorMax = Math.max(max(colorValues) ?? 0, 1);
      colorScale = scaleQuantize<string>().domain([0, colorMax]).range(config.palette);
    }

    const radiusValues = parsedItems
      .map((entry) => getMetricValue(entry.item, selectedRadiusField.value, config.defaultMotivation))
      .filter((value): value is number => value !== null);

    const radiusExtent = extent(radiusValues);
    const radiusMin = radiusExtent[0] ?? 0;
    const radiusMax = radiusExtent[1] ?? radiusMin + 1;
    const radiusScale = scaleLinear()
      .domain(radiusMin === radiusMax ? [radiusMin, radiusMin + 1] : [radiusMin, radiusMax])
      .range([config.minRadius, config.maxRadius]);

    const previousItems = new Map(graphItems.value.map((item) => [item.id, item]));

    const nextItems: GraphItem[] = parsedItems.map(({ item, dueMs }) => {
      const timeDiffMs = dueMs - nowMs;
      const clampedDiff = clamp(timeDiffMs, -rangeMs, rangeMs);
      const marker =
        timeDiffMs < -rangeMs
          ? 'triangle-left'
          : timeDiffMs > rangeMs
            ? 'triangle-right'
            : 'circle';

      const yValue = getMetricValue(item, selectedYField.value, config.defaultMotivation) ?? 0;
      const radiusMetric = getMetricValue(item, selectedRadiusField.value, config.defaultMotivation);
      const colorMetric = getMetricValue(item, selectedColorField.value, config.defaultMotivation);
      const previous = previousItems.get(item.id);

      let color = config.neutralColor;
      if (selectedColorField.value === 'status') {
        color = STATUS_COLOR[item.status];
      } else if (colorScale && colorMetric !== null) {
        color = colorScale(colorMetric);
      }

      let radius = config.defaultRadius;
      if (selectedRadiusField.value === 'status') {
        radius = radiusScale(STATUS_VALUE[item.status]);
      } else if (selectedRadiusField.value !== 'none' && radiusMetric !== null) {
        radius = radiusScale(radiusMetric);
      }

      const targetX = xScale(clampedDiff);
      const targetY = yScale(yValue);
      const safeTarget = {
        x: Number.isFinite(targetX) ? targetX : xScale(0),
        y: Number.isFinite(targetY) ? targetY : plotBottom,
      };

      if (!Number.isFinite(targetX) || !Number.isFinite(targetY)) {
        console.error('[useGraph] Non-finite target coordinate detected', {
          id: item.id,
          due: item.due,
          yValue,
          targetX,
          targetY,
        });
      }

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
        priority: getPriorityScore(item, timeDiffMs),
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

    for (let tick = 0; tick < config.tickCount; tick += 1) {
      simulation.tick();
    }

    nextItems.forEach((item) => {
      item.point = {
        x: clamp(item.x ?? item.target.x, plotLeft, plotRight),
        y: clamp(item.y ?? item.target.y, plotTop, plotBottom),
      };
    });

    const invalidDueCount = visibleItems.length - parsedItems.length;

    graphItems.value = nextItems;
    graphGroups.value = groupingEnabled.value ? aggregateGroups(nextItems) : singleItemGroups(nextItems);
    debugStats.value = {
      inputItems: rawItems.length,
      visibleItems: visibleItems.length,
      plottedItems: nextItems.length,
      invalidDueCount,
    };
    warnings.value =
      [
        missingYItems.length > 0
          ? `Y-axis field "${selectedYField.value}" is missing on ${missingYItems.length} item(s). Fallback values were used.`
          : null,
        invalidDueCount > 0
          ? `Skipped ${invalidDueCount} item(s) due to invalid due timestamps.`
          : null,
      ].filter((warning): warning is string => warning !== null);

    console.log('[useGraph] recompute summary', {
      inputItems: rawItems.length,
      visibleItems: visibleItems.length,
      plottedItems: nextItems.length,
      grouped: groupingEnabled.value,
      invalidDueCount,
    });
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
    yTicks.value = buildYTicks(yScale, yMax, selectedYField.value);
  };

  const setDimensions = (width: number, height: number): void => {
    const safeWidth = Math.max(Math.round(width), 320);
    const safeHeight = Math.max(Math.round(height), 420);

    if (safeWidth === layout.value.width && safeHeight === layout.value.height) {
      return;
    }

    layout.value = {
      ...layout.value,
      width: safeWidth,
      height: safeHeight,
    };
    recompute();
  };

  const updateData = (items: Item[]): void => {
    rawItems = items;
    recompute();
  };

  const refreshNow = (): void => {
    recompute();
  };

  const destroy = (): void => {
    simulation?.stop();
    simulation = null;
  };

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