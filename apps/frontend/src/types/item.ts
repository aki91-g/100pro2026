/**
 * Item Type Definitions
 * 
 * Centralized types for Item, UUID, and related structures.
 * Matches both the Rust backend (Tauri) and Hono/Node.js backend (web) Item struct exactly.
 */

export type UUID = string;

/**
 * Matches the Rust backend and Hono backend Item struct exactly.
 * Ensure the status strings match your backend's TaskStatus enum/type variants.
 */
export type Item = {
  id: UUID;
  user_id: UUID; // Changed to mandatory, matching Rust backend
  sync_status: "synced" | "local_only" | "modified";
  title: string;
  description: string | null;
  status: "backlog" | "todo" | "inprogress" | "done";
  due: string;
  duration_minutes: number | null;
  motivation: number | null;
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