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
  const envMode = import.meta.env.VITE_API_MODE;
  if (envMode) {
    if (envMode !== 'tauri' && envMode !== 'hono') {
      const errorMsg = `Invalid VITE_API_MODE: "${envMode}". Must be "tauri" or "hono".`;
      console.error(errorMsg);
      throw new Error(errorMsg);
    }
    cachedApiMode = envMode as ApiMode;
    return cachedApiMode;
  }

  // Default to Hono for browser environments
  cachedApiMode = 'hono';
  return cachedApiMode;
}

/**
 * Check if we're using the Tauri backend
 */
export function usesTauriBackend(): boolean {
  return getApiMode() === 'tauri';
}

/**
 * Check if we're using the Hono backend
 */
export function usesHonoBackend(): boolean {
  return getApiMode() === 'hono';
}
