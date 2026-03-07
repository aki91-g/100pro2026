import { invoke } from "@tauri-apps/api/core";
import type { Item } from "@/services/itemService";

export interface HonoHelloResponse {
  message: string;
  timestamp: string;
}

export async function isDevMode(): Promise<boolean> {
  return invoke<boolean>("is_dev", {});
}

export async function fetchActiveItemsApi(): Promise<Item[]> {
  return invoke<Item[]>("get_active_items", {});
}

export async function fetchArchivedItemsApi(): Promise<Item[]> {
  return invoke<Item[]>("get_archived_items", {});
}

export async function fetchDeletedItemsApi(): Promise<Item[]> {
  return invoke<Item[]>("get_deleted_items", {});
}

export async function syncItemsApi(): Promise<number> {
  return invoke<number>("sync_items", {});
}

export async function createItemApi(payload: {
  title: string;
  motivation: number;
  due?: string | null;
  durationMinutes?: number | null;
}): Promise<string> {
  return invoke<string>("create_item", payload);
}

export async function updateItemStatusApi(id: string, status: Item["status"]): Promise<void> {
  await invoke("update_item_status", { id, status });
}

export async function archiveItemApi(id: string): Promise<void> {
  await invoke("archive_item", { id });
}

export async function softDeleteItemApi(id: string): Promise<void> {
  await invoke("soft_delete_item", { id });
}

export async function seedDatabaseApi(): Promise<void> {
  await invoke("debug_seed_data", {});
}

export async function resetDatabaseApi(): Promise<void> {
  await invoke("debug_reset_db", {});
}

export async function migrateNullUserItemsApi(assignToCurrentUser: boolean): Promise<number> {
  return invoke<number>("debug_migrate_null_users", { assignToCurrentUser });
}

export async function fetchHonoHelloApi(): Promise<HonoHelloResponse> {
  // Use environment variable or fallback to localhost:3000
  const honoBaseUrl = import.meta.env.VITE_HONO_BASE_URL || "http://localhost:3000";
  const url = `${honoBaseUrl}/api/hello`;
  
  // Create abort controller with 5 second timeout
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), 5000);

  try {
    const response = await fetch(url, {
      signal: controller.signal,
    });

    if (!response.ok) {
      throw new Error("Network response was not ok");
    }

    return response.json() as Promise<HonoHelloResponse>;
  } finally {
    clearTimeout(timeout);
  }
}