import type { Item } from './item';

/**
 * graph rendering data types
 */
export interface Point {
  x: number;
  y: number;
}

/**
 * metadata for rendering an item on the graph (position, color, etc.)
 * This is derived from the base Item data but includes additional properties needed for visualization.
 */
export interface GraphItem extends Item {
  point: Point;
  radius: number;
  color: string;
}

/**
 * range & grid lines configuration for the graph
 */
export interface GraphConfig {
  padding: {
    top: number;
    right: number;
    bottom: number;
    left: number;
  };
  gridLines: number[]; // [0, 25, 50, 75, 100] など
  defaultMotivation: number;
}

/**
 * interaction state for the graph (hovered/selected item, zoom range, etc.)
 */
export interface GraphState {
  hoveredItemId: string | null;
  selectedItemId: string | null;
  viewRange: {
    start: Date;
    end: Date;
  } | null;
}