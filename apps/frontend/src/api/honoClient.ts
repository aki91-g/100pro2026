import type { Item } from '@/types/item';

const DEFAULT_RENDER_BASE_URL = 'https://taskgraph-100program9-server.onrender.com';

function resolveHonoBaseUrl(explicitBaseUrl?: string): string {
  if (explicitBaseUrl && explicitBaseUrl.trim().length > 0) {
    return explicitBaseUrl.trim().replace(/\/+$/, '');
  }

  const envBaseUrl = (import.meta.env.VITE_HONO_BASE_URL as string | undefined)?.trim();
  if (envBaseUrl && envBaseUrl.length > 0) {
    return envBaseUrl.replace(/\/+$/, '');
  }

  if (typeof window !== 'undefined') {
    const host = window.location.hostname.toLowerCase();
    if (host === 'localhost' || host === '127.0.0.1') {
      return 'http://localhost:10000';
    }
  }

  return DEFAULT_RENDER_BASE_URL;
}


export interface HonoItemsClient {
  getActiveItems(): Promise<Item[]>;
  getArchivedItems(): Promise<Item[]>;
  getDeletedItems(): Promise<Item[]>;
  createItem(payload: CreateItemPayload): Promise<string>;
  updateItem(payload: UpdateItemPayload): Promise<void>;
  updateItem(payload: UpdateItemPayload): Promise<void>;
  updateItemStatus(id: string, status: Item['status']): Promise<void>;
  archiveItem(id: string): Promise<void>;
  deleteItem(id: string): Promise<void>;
  softDeleteItem(id: string): Promise<void>;
  syncItems(): Promise<number>;
}

export interface CreateItemPayload {
  title: string;
  description?: string | null;
  motivation: number | null;
  due: string;
  durationMinutes?: number | null;
}

export interface UpdateItemPayload {
  id: string;
  title: string;
  description: string | null;
  description?: string | null;
  motivation: number | null;
  due: string;
  durationMinutes?: number | null;
}

export interface UpdateItemPayload {
  id: string;
  title: string;
  description: string | null;
  motivation: number | null;
  due: string;
  durationMinutes?: number | null;
}

export class HonoClient implements HonoItemsClient {
  private baseUrl: string;
  private tokenGetter: (() => string | null) | null = null;

  constructor(baseUrl?: string) {
    this.baseUrl = resolveHonoBaseUrl(baseUrl);
  }

  private normalizeStatus(raw: unknown): Item['status'] {
    if (raw === 'backlog' || raw === 'todo' || raw === 'inprogress' || raw === 'done') {
      return raw;
    }
    return 'todo';
  }

  private normalizeNumber(raw: unknown): number | null {
    if (typeof raw === 'number' && Number.isFinite(raw)) return raw;
    if (typeof raw === 'string' && raw.trim().length > 0) {
      const parsed = Number(raw);
      if (Number.isFinite(parsed)) return parsed;
    }
    return null;
  }

  private normalizeDue(raw: unknown): string {
    if (typeof raw !== 'string') return '';
    const trimmed = raw.trim();
    if (trimmed.length === 0) return '';
    const ms = Date.parse(trimmed);
    if (!Number.isFinite(ms)) return trimmed;
    return new Date(ms).toISOString();
  }

  private normalizeItem(raw: unknown): Item {
    const row = (raw ?? {}) as Record<string, unknown>;

    return {
      id: String(row.id ?? ''),
      user_id: String(row.user_id ?? ''),
      sync_status:
        row.sync_status === 'synced' || row.sync_status === 'local_only' || row.sync_status === 'modified'
          ? row.sync_status
          : 'local_only',
      title: typeof row.title === 'string' ? row.title : '',
      description: typeof row.description === 'string' ? row.description : null,
      status: this.normalizeStatus(row.status),
      due: this.normalizeDue(row.due),
      duration_minutes: this.normalizeNumber(row.duration_minutes),
      motivation: this.normalizeNumber(row.motivation),
      is_archived: Boolean(row.is_archived),
      created_at: typeof row.created_at === 'string' ? row.created_at : '',
      updated_at: typeof row.updated_at === 'string' ? row.updated_at : '',
      deleted_at: typeof row.deleted_at === 'string' ? row.deleted_at : null,
    };
  }

  private normalizeItems(raw: unknown): Item[] {
    if (!Array.isArray(raw)) return [];
    const normalized = raw.map((entry) => this.normalizeItem(entry));
    const invalidDueCount = normalized.filter((item) => Number.isNaN(Date.parse(item.due))).length;
    if (invalidDueCount > 0) {
      console.warn(`[HonoClient] Received ${invalidDueCount} item(s) with invalid due timestamps.`);
    }
    return normalized;
  }

  /**
   * Set the token getter function to avoid Pinia instance errors
   */
  setTokenGetter(fn: () => string | null): void {
    this.tokenGetter = fn;
  }

  /**
   * Get headers with Authorization token if available
   */
  private getHeaders(): HeadersInit {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };

    // Get token via the injected function
    if (this.tokenGetter) {
      try {
        const token = this.tokenGetter();
        if (token) {
          headers['Authorization'] = `Bearer ${token}`;
        }
      } catch (error) {
        // Keep request functional even if store is not ready yet.
        console.warn('Token getter failed. Sending request without Authorization header.');
      }
    }

    return headers;
  }

  /**
   * Generic GET request
   */
  async get(path: string): Promise<Response> {
    const response = await fetch(`${this.baseUrl}${path}`, {
      method: 'GET',
      headers: this.getHeaders(),
    });
    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(`GET ${path} failed: ${errorData.error || response.statusText}`);
    }
    return response;
  }

  /**
   * Generic HTTP request with configurable method
   */
  async request(path: string, body?: unknown, method: string = 'POST'): Promise<Response> {
    const url = `${this.baseUrl}/${path}`.replace(/\/+/g, '/').replace(':/', '://');
    const response = await fetch(url, {
      method,
      headers: this.getHeaders(),
      body: body ? JSON.stringify(body) : undefined,
    });
    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(`${method} ${path} failed: ${errorData.error || response.statusText}`);
    }
    return response;
  }

  /**
   * POST request wrapper
   */
  async post(path: string, body?: unknown): Promise<Response> {
    return this.request(path, body, 'POST');
  }

  /**
   * PATCH request wrapper
   */
  async patch(path: string, body?: unknown): Promise<Response> {
    return this.request(path, body, 'PATCH');
  }

  /**
   * DELETE request wrapper
   */
  async delete(path: string, body?: unknown): Promise<Response> {
    return this.request(path, body, 'DELETE');
  }

  async getActiveItems(): Promise<Item[]> {
    const response = await this.get('/api/items/active');
    const data = await response.json();
    return this.normalizeItems(data);
  }

  async getArchivedItems(): Promise<Item[]> {
    const response = await this.get('/api/items/archived');
    const data = await response.json();
    return this.normalizeItems(data);
  }

  async getDeletedItems(): Promise<Item[]> {
    const response = await this.get('/api/items/deleted');
    const data = await response.json();
    return this.normalizeItems(data);
  }

  async createItem(payload: CreateItemPayload): Promise<string> {
    const response = await this.post('/api/items', payload);
    const data = await response.json();
    return data.id;
  }

  async updateItem(payload: UpdateItemPayload): Promise<void> {
    await this.patch(`/api/items/${payload.id}`, {
      title: payload.title,
      description: payload.description,
      motivation: payload.motivation,
      due: payload.due,
      durationMinutes: payload.durationMinutes ?? null,
    });
  }

  async updateItemStatus(id: string, status: Item['status']): Promise<void> {
    await this.patch(`/api/items/${id}/status`, { status });
  }

  async archiveItem(id: string): Promise<void> {
    await this.post(`/api/items/${id}/archive`);
  }

  async deleteItem(id: string): Promise<void> {
    await this.delete(`/api/items/${id}`);
  }

  async softDeleteItem(id: string): Promise<void> {
    await this.deleteItem(id);
  }

  async syncItems(): Promise<number> {
    const response = await this.post('/api/items/sync');
    const data = await response.json();
    return data.count || 0;
  }
}

// Export singleton instance
export const honoClient = new HonoClient();
