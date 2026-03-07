/**
 * Hono RPC Client (Placeholder)
 * 
 * This is a placeholder for future Hono backend integration using hc (Hono Client).
 * When the Hono backend is ready, you can use type-safe RPC calls like:
 * 
 * import { hc } from 'hono/client'
 * import type { AppType } from '../backend/src/index'
 * 
 * const client = hc<AppType>('/api')
 * const res = await client.items.$get()
 */

import type { Item } from '@/types/item';

export interface HonoItemsClient {
  getActiveItems(): Promise<Item[]>;
  getArchivedItems(): Promise<Item[]>;
  getDeletedItems(): Promise<Item[]>;
  createItem(payload: CreateItemPayload): Promise<string>;
  updateItemStatus(id: string, status: Item['status']): Promise<void>;
  archiveItem(id: string): Promise<void>;
  softDeleteItem(id: string): Promise<void>;
  syncItems(): Promise<number>;
}

export interface CreateItemPayload {
  title: string;
  motivation: number;
  due?: string | null;
  durationMinutes?: number | null;
}

/**
 * Placeholder Hono client implementation
 * Replace this with actual Hono RPC client when backend is ready
 */
export class HonoClient implements HonoItemsClient {
  private baseUrl: string;

  constructor(baseUrl?: string) {
    this.baseUrl = baseUrl || import.meta.env.VITE_HONO_BASE_URL || 'http://localhost:3000';
  }

  /**
   * Generic GET request
   */
  async get(path: string): Promise<Response> {
    const response = await fetch(`${this.baseUrl}${path}`);
    if (!response.ok) throw new Error(`GET ${path} failed`);
    return response;
  }

  /**
   * Generic POST request
   */
  async post(path: string, body?: unknown): Promise<Response> {
    const response = await fetch(`${this.baseUrl}${path}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: body ? JSON.stringify(body) : undefined,
    });
    if (!response.ok) throw new Error(`POST ${path} failed`);
    return response;
  }

  async getActiveItems(): Promise<Item[]> {
    const response = await fetch(`${this.baseUrl}/api/items/active`);
    if (!response.ok) throw new Error('Failed to fetch active items');
    return response.json();
  }

  async getArchivedItems(): Promise<Item[]> {
    const response = await fetch(`${this.baseUrl}/api/items/archived`);
    if (!response.ok) throw new Error('Failed to fetch archived items');
    return response.json();
  }

  async getDeletedItems(): Promise<Item[]> {
    const response = await fetch(`${this.baseUrl}/api/items/deleted`);
    if (!response.ok) throw new Error('Failed to fetch deleted items');
    return response.json();
  }

  async createItem(payload: CreateItemPayload): Promise<string> {
    const response = await fetch(`${this.baseUrl}/api/items`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    if (!response.ok) throw new Error('Failed to create item');
    const data = await response.json();
    return data.id;
  }

  async updateItemStatus(id: string, status: Item['status']): Promise<void> {
    const response = await fetch(`${this.baseUrl}/api/items/${id}/status`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ status }),
    });
    if (!response.ok) throw new Error('Failed to update item status');
  }

  async archiveItem(id: string): Promise<void> {
    const response = await fetch(`${this.baseUrl}/api/items/${id}/archive`, {
      method: 'POST',
    });
    if (!response.ok) throw new Error('Failed to archive item');
  }

  async softDeleteItem(id: string): Promise<void> {
    const response = await fetch(`${this.baseUrl}/api/items/${id}`, {
      method: 'DELETE',
    });
    if (!response.ok) throw new Error('Failed to delete item');
  }

  async syncItems(): Promise<number> {
    const response = await fetch(`${this.baseUrl}/api/items/sync`, {
      method: 'POST',
    });
    if (!response.ok) throw new Error('Failed to sync items');
    const data = await response.json();
    return data.count || 0;
  }
}

// Export singleton instance
export const honoClient = new HonoClient();
