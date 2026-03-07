/**
 * API Configuration
 * Determines which backend to use (Tauri Local or Hono Remote)
 */

export type ApiMode = 'tauri' | 'hono';

/**
 * Gets the current API mode based on environment or runtime detection
 * Priority: environment variable > Tauri detection > default to Hono (browser)
 */
export function getApiMode(): ApiMode {
  // Check environment variable first
  if (import.meta.env.VITE_API_MODE === 'tauri') {
    return 'tauri';
  }
  if (import.meta.env.VITE_API_MODE === 'hono') {
    return 'hono';
  }

  // Check if we're running in Tauri runtime
  if ((window as any).__TAURI_INTERNALS__) {
    return 'tauri';
  }

  // Default to Hono for browser environments
  return 'hono';
}

/**
 * Check if we're in Tauri mode
 */
export function isTauriMode(): boolean {
  return getApiMode() === 'tauri';
}

/**
 * Check if we're in Hono mode
 */
export function isHonoMode(): boolean {
  return getApiMode() === 'hono';
}
