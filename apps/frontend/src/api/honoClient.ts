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

export class HonoClient implements HonoItemsClient {
  private baseUrl: string;
  private tokenGetter: (() => string | null) | null = null;

  constructor(baseUrl?: string) {
    this.baseUrl = baseUrl || import.meta.env.VITE_HONO_BASE_URL || 'http://localhost:3000';
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
    const response = await fetch(`${this.baseUrl}${path}`, {
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
    return response.json();
  }

  async getArchivedItems(): Promise<Item[]> {
    const response = await this.get('/api/items/archived');
    return response.json();
  }

  async getDeletedItems(): Promise<Item[]> {
    const response = await this.get('/api/items/deleted');
    return response.json();
  }

  async createItem(payload: CreateItemPayload): Promise<string> {
    const response = await this.post('/api/items', payload);
    const data = await response.json();
    return data.id;
  }

  async updateItemStatus(id: string, status: Item['status']): Promise<void> {
    await this.patch(`/api/items/${id}/status`, { status });
  }

  async archiveItem(id: string): Promise<void> {
    await this.post(`/api/items/${id}/archive`);
  }

  async softDeleteItem(id: string): Promise<void> {
    await this.delete(`/api/items/${id}`);
  }

  async syncItems(): Promise<number> {
    const response = await this.post('/api/items/sync');
    const data = await response.json();
    return data.count || 0;
  }
}

// Export singleton instance
export const honoClient = new HonoClient();
