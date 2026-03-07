import type { Item } from '@/types/item';
import { useUserStore } from '@/stores/user'; // ユーザー情報を取得するためにインポート

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

  constructor(baseUrl?: string) {
    this.baseUrl = baseUrl || import.meta.env.VITE_HONO_BASE_URL || 'http://localhost:3000';
  }

  /**
   * 共通のヘッダー取得メソッド
   */
  private getHeaders(): HeadersInit {
    const userStore = useUserStore();
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };

    // Supabaseのアクセストークンがあればヘッダーに載せる
    if (userStore.accessToken) {
      headers['Authorization'] = `Bearer ${userStore.accessToken}`;
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
   * Generic POST request
   */
  async post(path: string, body?: unknown, method: string = 'POST'): Promise<Response> {
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
    // Hono側が PATCH で待ち受けている場合は method を指定
    await this.post(`/api/items/${id}/status`, { status }, 'PATCH');
  }

  async archiveItem(id: string): Promise<void> {
    await this.post(`/api/items/${id}/archive`);
  }

  async softDeleteItem(id: string): Promise<void> {
    // 物理削除ではなく論理削除（deleted_at更新）なので、API設計に合わせて DELETE を使用
    await this.post(`/api/items/${id}`, undefined, 'DELETE');
  }

  async syncItems(): Promise<number> {
    const response = await this.post('/api/items/sync');
    const data = await response.json();
    return data.count || 0;
  }
}

// Export singleton instance
export const honoClient = new HonoClient();
