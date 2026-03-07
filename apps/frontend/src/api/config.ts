/**
 * API Configuration
 * Determines which backend to use (Tauri Local or Hono Remote)
 */

export type ApiMode = 'tauri' | 'hono';

let cachedApiMode: ApiMode | null = null;

/**
 * Gets the current API mode based on runtime detection and environment settings.
 *
 * Safety rule:
 * - If we are in a Tauri WebView, always use `tauri` mode.
 * - Outside Tauri, environment variable can force mode.
 * - Browser default is `hono`.
 */
export function getApiMode(): ApiMode {
  if (cachedApiMode) {
    return cachedApiMode;
  }

  const hasWindow = typeof window !== 'undefined';
  const runtime = hasWindow ? (window as any) : undefined;
  const isTauriRuntime = !!runtime?.__TAURI__ || !!runtime?.__TAURI_INTERNALS__;

  // Never fall back to Hono when running inside Tauri.
  if (isTauriRuntime) {
    cachedApiMode = 'tauri';
    return cachedApiMode;
  }

  // Environment override applies outside Tauri runtime.
  if (import.meta.env.VITE_API_MODE === 'tauri') {
    cachedApiMode = 'tauri';
    return cachedApiMode;
  }
  if (import.meta.env.VITE_API_MODE === 'hono') {
    cachedApiMode = 'hono';
    return cachedApiMode;
  }

  // Default to Hono for browser environments
  cachedApiMode = 'hono';
  return cachedApiMode;
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
