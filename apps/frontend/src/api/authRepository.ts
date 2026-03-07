import { invoke } from '@tauri-apps/api/core';
import { getApiMode } from './config';
import { honoClient } from './honoClient';

// Types
export interface LoginResponse {
  id: string;
  username: string;
}

export interface LocalSession {
  id: string;
  username: string;
  last_login: string | null;
  is_active: number;
}

/**
 * Abstract authentication repository interface
 */
export interface AuthRepository {
  login(email: string, password: string): Promise<LoginResponse>;
  logout(): Promise<void>;
  getActiveSession(): Promise<LocalSession | null>;
  autoLogin(): Promise<LocalSession | null>;
}

/**
 * Tauri implementation using Rust backend commands
 */
export class TauriAuthRepository implements AuthRepository {
  async login(email: string, password: string): Promise<LoginResponse> {
    return invoke<LoginResponse>('login', { email, password });
  }

  async logout(): Promise<void> {
    await invoke<void>('logout', {});
  }

  async getActiveSession(): Promise<LocalSession | null> {
    return invoke<LocalSession | null>('get_active_session', {});
  }

  async autoLogin(): Promise<LocalSession | null> {
    return invoke<LocalSession | null>('auto_login', {});
  }
}

/**
 * Hono implementation using REST API
 */
export class HonoAuthRepository implements AuthRepository {
  async login(email: string, password: string): Promise<LoginResponse> {
    const response = await honoClient.post('/api/auth/login', {
      email,
      password,
    });
    return response.json();
  }

  async logout(): Promise<void> {
    await honoClient.post('/api/auth/logout', {});
  }

  async getActiveSession(): Promise<LocalSession | null> {
    const response = await honoClient.get('/api/auth/session');
    return response.json();
  }

  async autoLogin(): Promise<LocalSession | null> {
    const response = await honoClient.post('/api/auth/auto-login', {});
    return response.json();
  }
}

/**
 * Factory function to create the appropriate repository based on the API mode
 */
export function createAuthRepository(): AuthRepository {
  const mode = getApiMode();

  if (mode === 'tauri') {
    return new TauriAuthRepository();
  }

  return new HonoAuthRepository();
}

// Export a singleton instance
export const authRepository = createAuthRepository();
