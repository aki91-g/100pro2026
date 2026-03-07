/**
 * API Configuration
 * Determines which backend to use (Tauri Local or Hono Remote)
 */

export type ApiMode = 'tauri' | 'hono';

/**
 * Gets the current API mode based on environment or runtime detection
 * For now, we default to Tauri since Hono backend isn't implemented yet
 */
export function getApiMode(): ApiMode {
  // Check if we're running in Tauri
  if ((window as any).__TAURI_INTERNALS__) {
    return 'tauri';
  }

  // Future: Check environment variable for Hono mode
  // if (import.meta.env.VITE_API_MODE === 'hono') {
  //   return 'hono';
  // }

  return 'tauri';
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
