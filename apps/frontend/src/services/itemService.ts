import { invoke } from "@tauri-apps/api/core";

/**
 * Matches the Rust Item struct exactly.
 * Ensure the status strings match your Rust TaskStatus enum variants.
 */
export type Item = {
  id: string;
  user_id: string | null;
  title: string;
  description: string | null;
  status: "Backlog" | "Todo" | "InProgress" | "Done";
  due: string | null;
  duration_minutes: number | null;
  motivation: number;
  is_archived: boolean;
  created_at: string;
  updated_at: string;
  deleted_at: string | null;
};

/**
 * Result object for UI state updates
 */
export type RefreshResult = {
  active: Item[];
  archived: Item[];
  deleted: Item[];
};

/**
 * Fetches all items from the local SQLite database, categorized by state.
 */
export async function refreshItems(): Promise<RefreshResult> {
  try {
    const [active, archived, deleted] = await Promise.all([
      invoke<Item[]>("get_active_items"),
      invoke<Item[]>("get_archived_items"),
      invoke<Item[]>("get_deleted_items"),
    ]);

    return { active, archived, deleted };
  } catch (err) {
    console.error("Failed to fetch items from local database:", err);
    throw err;
  }
}

/**
 * Triggers the Cloud -> Local sync engine in Rust.
 * 1. Pushes remote changes to local SQLite via UPSERT.
 * 2. Mirrors hard deletions from remote.
 * 3. Refreshes the local view.
 */
export async function syncAndRefresh(): Promise<{ count: number; data: RefreshResult }> {
  try {
    // 1. Run the Rust sync logic
    const count = await invoke<number>("sync_items");
    
    // 2. Immediately fetch the updated local state
    const data = await refreshItems();
    
    return { count, data };
  } catch (err) {
    console.error("Sync operation failed:", err);
    throw err;
  }
}

/**
 * Debug only: Wipes both local and remote databases.
 */
export async function hardResetAll(): Promise<void> {
  try {
    await invoke("debug_full_wipe_items");
    console.warn("Full database wipe completed.");
  } catch (err) {
    console.error("Hard reset failed:", err);
    throw err;
  }
}