import { invoke } from '@tauri-apps/api/core';
import { usesHonoBackend, usesTauriBackend } from './config';
import { honoClient } from './honoClient';

// Types
export interface LoginResponse {
  id: string;
  username: string;
  access_token: string;
  refresh_token?: string;
  expires_at?: number;
}

export interface SignUpResponse {
  id: string;
  username: string;
  access_token?: string | null;
  refresh_token?: string | null;
  expires_at?: number | null;
}

export interface LocalSession {
  id: string;
  username: string;
  access_token?: string | null;
  last_login: string | null;
  is_active: number;
}

/**
 * Abstract authentication repository interface
 */
export interface AuthRepository {
  signUp(email: string, password: string, username: string): Promise<SignUpResponse>;
  login(email: string, password: string): Promise<LoginResponse>;
  logout(): Promise<void>;
  getActiveSession(): Promise<LocalSession | null>;
  autoLogin(): Promise<LocalSession | null>;
}

/**
 * Tauri implementation using Rust backend commands
 */
export class TauriAuthRepository implements AuthRepository {
  async signUp(email: string, password: string, username: string): Promise<SignUpResponse> {
    const normalizedEmail = email.trim();
    const normalizedUsername = username.trim();
    if (!normalizedEmail || !password || !normalizedUsername) {
      throw new Error('email, password, and username are required');
    }

    try {
      return await invoke<SignUpResponse>('register_local_user', {
        email: normalizedEmail,
        password,
        username: normalizedUsername,
      });
    } catch (error) {
      const message = String(error ?? '');
      if (message.includes('OFFLINE_REQUIRED_FOR_SIGNUP')) {
        throw new Error('OFFLINE_REQUIRED_FOR_SIGNUP');
      }
      throw error;
    }
  }

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
  async signUp(email: string, password: string, username: string): Promise<SignUpResponse> {
    const response = await honoClient.post('/api/auth/signup', {
      email,
      password,
      username,
    });
    return response.json();
  }

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
  if (usesTauriBackend()) {
    return new TauriAuthRepository();
  }

  if (usesHonoBackend()) {
    return new HonoAuthRepository();
  }

  throw new Error('Unsupported API mode for auth repository');
}

// Export a singleton instance
export const authRepository = createAuthRepository();
