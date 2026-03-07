/**
 * API Configuration
 * Determines which backend to use (Tauri Local or Hono Remote)
 */

export type ApiMode = 'tauri' | 'hono';

/**
 * Gets the current API mode based on runtime detection and environment settings.
 *
 * Safety rule:
 * - If we are in a Tauri WebView, always use `tauri` mode.
 * - Outside Tauri, environment variable can force mode.
 * - Browser default is `hono`.
 */
export function getApiMode(): ApiMode {
  const hasWindow = typeof window !== 'undefined';
  const runtime = hasWindow ? (window as any) : undefined;
  const isTauriRuntime = !!runtime?.__TAURI__ || !!runtime?.__TAURI_INTERNALS__;

  // Never fall back to Hono when running inside Tauri.
  if (isTauriRuntime) {
    return 'tauri';
  }

  // Environment override applies outside Tauri runtime.
  if (import.meta.env.VITE_API_MODE === 'tauri') {
    return 'tauri';
  }
  if (import.meta.env.VITE_API_MODE === 'hono') {
    return 'hono';
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
