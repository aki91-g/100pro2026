import type { Item } from './item';

export type GraphTimeRangeKey = '1d' | '3d' | '1w' | '2w' | '1m';
export type GraphAxisField = 'duration_minutes' | 'motivation' | 'status';
export type GraphVisualField = 'none' | 'duration_minutes' | 'motivation' | 'status';
export type GraphMarker = 'circle' | 'triangle-left' | 'triangle-right';

export interface SelectOption<T extends string> {
  label: string;
  value: T;
}

export interface Point {
  x: number;
  y: number;
}

export interface GraphTick {
  position: number;
  label: string;
}

export interface GraphItem extends Item {
  point: Point;
  target: Point;
  radius: number;
  color: string;
  marker: GraphMarker;
  timeDiffMs: number;
  yValue: number;
  isOverdue: boolean;
  isClamped: boolean;
  priority: number;
  index?: number;
  x?: number;
  y?: number;
  vx?: number;
  vy?: number;
}

export interface GraphGroup {
  key: string;
  items: GraphItem[];
  point: Point;
  radius: number;
  color: string;
  marker: GraphMarker;
  label: string;
  representative: GraphItem;
}

export interface GraphConfig {
  padding: {
    top: number;
    right: number;
    bottom: number;
    left: number;
  };
  defaultMotivation: number;
  defaultRadius: number;
  minRadius: number;
  maxRadius: number;
  collisionPadding: number;
  groupDistance: number;
  tickCount: number;
  timeRanges: Record<GraphTimeRangeKey, number>;
  palette: string[];
  neutralColor: string;
  overdueShade: string;
}

export interface GraphLayout {
  width: number;
  height: number;
  plotLeft: number;
  plotRight: number;
  plotTop: number;
  plotBottom: number;
  innerWidth: number;
  innerHeight: number;
  originX: number;
  yMax: number;
  nowMs: number;
}