/// <reference types="vite/client" />

interface ImportMetaEnv {
	readonly VITE_API_MODE?: 'tauri' | 'hono';
	readonly VITE_HONO_BASE_URL: string;
	readonly SUPABASE_URL?: string;
	readonly SUPABASE_ANON_KEY?: string;
}

interface ImportMeta {
	readonly env: ImportMetaEnv;
}