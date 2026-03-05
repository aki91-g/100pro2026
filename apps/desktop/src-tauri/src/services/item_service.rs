use std::{sync::Arc};
use chrono::{DateTime, Utc, Timelike};
use uuid::Uuid;
use tauri::{AppHandle, Emitter};
use tracing::{info, error, warn};
use tokio::sync::RwLock;

use crate::{
    error::AppResult,
    models::item::{Item, TaskStatus},
    repositories::item_repo::ItemRepository,
};

#[derive(serde::Serialize, Clone)]
struct SyncEvent {
    id: Uuid,
    status: String,
    message: Option<String>,
}

#[derive(serde::Serialize, Clone)]
struct RemoteCatchupEvent {
    user_id: Uuid,
    synced_count: usize,
}

pub struct ItemService {
    repo: Arc<dyn ItemRepository>,
    // We use RwLock so we can plug in Postgres later without freezing the app
    remote: RwLock<Option<Arc<dyn ItemRepository>>>,
    app_handle: AppHandle,
}

impl ItemService {
    pub fn new(
        repo: Arc<dyn ItemRepository>, 
        remote: Option<Arc<dyn ItemRepository>>, 
        app_handle: AppHandle
    ) -> Self {
        Self { 
            repo, 
            remote: RwLock::new(remote), 
            app_handle 
        }
    }

    /// This allows the background thread to "enable" Postgres once it connects
    pub async fn set_remote(&self, remote_repo: Arc<dyn ItemRepository>) {
        let mut w = self.remote.write().await;
        *w = Some(remote_repo);
        info!("Postgres sync repository has been activated.");
    }

    /// Truncates DateTime to second-level precision to avoid floating-point mismatches.
    /// SQLite may lose nanosecond precision, so we normalize both timestamps before comparing.
    fn normalize_timestamp(ts: DateTime<Utc>) -> DateTime<Utc> {
        ts.with_nanosecond(0).unwrap_or(ts)
    }

    /// Updates local sync status with error recovery.
    /// If the update fails, logs a warning but doesn't break the operation.
    async fn mark_local_sync_status(&self, user_id: Uuid, id: Uuid, sync_status: &str) {
        if let Err(e) = self.repo.update_sync_status(user_id, id, sync_status).await {
            warn!(
                "Failed to set local sync_status '{}' for {}: {}",
                sync_status,
                id,
                e
            );
        }
    }

    // --- READ OPERATIONS ---
    pub async fn get_active_items(&self, user_id: Uuid) -> AppResult<Vec<Item>> {
        self.repo.get_active_items(user_id).await
    }

    pub async fn get_archived_items(&self, user_id: Uuid) -> AppResult<Vec<Item>> {
        self.repo.get_archived_items(user_id).await
    }

    pub async fn get_deleted_items(&self, user_id: Uuid) -> AppResult<Vec<Item>> {
        self.repo.get_deleted_items(user_id).await
    }

    // --- CLAIM OFFLINE ITEMS ---
    /// Claims any legacy items with NULL `user_id`.
    /// With `user_id` now mandatory, this is typically a no-op but kept for compatibility.
    pub async fn claim_offline_items(&self, user_id: Uuid) -> AppResult<usize> {
        let local_count = self.repo.claim_offline_items(user_id).await?;
        info!("Claimed {} offline items locally for user {}", local_count, user_id);

        let remote_lock = self.remote.read().await;
        if let Some(ref remote_repo) = *remote_lock {
            let remote_repo = remote_repo.clone();
            tokio::spawn(async move {
                match remote_repo.claim_offline_items(user_id).await {
                    Ok(count) => {
                        info!("Claimed {} offline items remotely for user {}", count, user_id);
                    }
                    Err(e) => {
                        error!("Failed to claim offline items remotely: {:?}", e);
                    }
                }
            });
        }

        Ok(local_count)
    }

    /// Pull items from remote and sync to local database.
    ///
    /// STATE MACHINE (Pull Operations):
    /// 1. For items existing in Postgres but NOT in SQLite ("born in remote"):\n    ///    - Create in local DB with default sync_status = 'local_only'
    ///    - Immediately mark as 'synced' to indicate successful pull
    ///    - This prevents re-upload, ensuring consistency
    ///
    /// 2. For items already in local DB:
    ///    - Update with remote state via upsert
    ///    - Mark as 'synced' (item was previously synced or is being refreshed)
    ///
    /// 3. Local-only items are PRESERVED:
    ///    - Items with sync_status='local_only' or 'modified' are not overwritten
    ///    - They will sync on next push_local_to_remote()
    ///
    /// TIMESTAMP HANDLING:
    /// - Timestamps normalized to second-precision to avoid floating-point mismatches
    /// - SQLite may lose nanosecond precision compared to Postgres
    ///
    /// Returns count of items successfully pulled and synced locally.
    pub async fn sync_items(&self, user_id: Uuid) -> AppResult<usize> {
        let remote_repo = {
            let remote_lock = self.remote.read().await;
            match &*remote_lock {
                Some(repo) => repo.clone(),
                None => {
                    return Err(crate::error::AppError::InvalidInput(
                        "Remote repository is not active".into(),
                    ));
                }
            }
        };

        let remote_items = remote_repo.get_all_items(user_id).await?;
        let local_items = self.repo.get_all_items(user_id).await?;
        
        // Build a map of local item IDs for fast lookup
        let local_ids: std::collections::HashSet<Uuid> = local_items.iter().map(|item| item.id).collect();
        
        let mut processed_count: usize = 0;

        for remote_item in remote_items {
            // Check if item exists locally
            let exists_locally = local_ids.contains(&remote_item.id);
            
            let motivation = i8::try_from(remote_item.motivation).map_err(
                |_| crate::error::AppError::InvalidInput(format!(
                    "Invalid motivation value for item {}: {}", 
                    remote_item.id, remote_item.motivation
                ))
            )?;

            // Create or upsert the item in local DB
            self.repo
                .create_item(
                    user_id,
                    remote_item.id,
                    remote_item.title.clone(),
                    motivation,
                    remote_item.due,
                    remote_item.duration_minutes,
                )
                .await?;

            // Update item status using enum's as_str() method for safety
            self.repo.update_item_status(user_id, remote_item.id, remote_item.status).await?;

            // Sync description, permissions, and metadata
            self.repo
                .update_item_details(
                    user_id,
                    remote_item.id,
                    remote_item.title,
                    remote_item.description,
                    remote_item.due,
                    remote_item.duration_minutes,
                    motivation,
                )
                .await?;

            // Sync archived state: only call if different from default
            if remote_item.is_archived {
                self.repo.archive_item(user_id, remote_item.id).await?;
            }

            // Sync deleted state: mark soft-deleted items
            if remote_item.deleted_at.is_some() {
                self.repo.soft_delete_item(user_id, remote_item.id).await?;
            }

            // CRITICAL: Mark pulled items as 'synced'
            // This prevents them from being re-uploaded, avoiding sync loops.
            // For items 'born in remote' (not existing locally), this marks them as downloaded.
            // For items already local, this marks them as in-sync with remote.
            if let Err(e) = self.repo.update_sync_status(user_id, remote_item.id, "synced").await {
                error!("Failed to mark pulled item {} as synced: {:?}. Item will retry on next pull.", remote_item.id, e);
            } else if !exists_locally {
                info!("Pulled item from remote (born in remote): {}", remote_item.id);
            }

            processed_count += 1;
        }

        // Note: Local-only items are preserved (they remain as-is).
        // These are typically items created while offline or not yet synced to remote.
        // They will naturally progress to 'synced' when pushed to remote on next sync_local_to_remote() call.

        Ok(processed_count)
    }

    /// Push local items to remote and sync their state.
    ///
    /// DIRECTIONAL SYNC (Push Only Unsync'd Items):
    /// - Only pushes items where sync_status is 'local_only' or 'modified'
    /// - Items already synced ('synced') are skipped (not re-pushed)
    /// - This reduces unnecessary remote writes and prevents re-sync loops
    ///
    /// STATE MACHINE (Push Operations):
    /// 1. For each unsync'd local item (local_only or modified):
    ///    - Create or update in remote DB (send title, details, status)
    ///    - Update archived/deleted state in remote
    ///
    /// 2. After successful remote upsert:
    ///    - Mark remote item sync_status = 'synced'
    ///    - Mark local item sync_status = 'synced'
    ///    - Both now consistent and won't be re-synced
    ///
    /// 3. On sync status update failure:
    ///    - Log error but continue (don't fail whole batch)
    ///    - Item will be retried on next push attempt (status unchanged)
    ///
    /// TIMESTAMP HANDLING:
    /// - Timestamps normalized to second-precision before remote operations
    /// - Avoids floating-point mismatches between SQLite and Postgres
    ///
    /// EVENTUAL CONSISTENCY:
    /// - If local sync_status update fails after remote succeeds:
    ///   Next pull will sync the item back as 'synced', achieving consistency
    ///
    /// Returns count of items successfully pushed and synced to remote.
    pub async fn sync_local_to_remote(&self, user_id: Uuid) -> AppResult<usize> {
        let remote_repo = {
            let remote_lock = self.remote.read().await;
            match &*remote_lock {
                Some(repo) => repo.clone(),
                None => {
                    return Err(crate::error::AppError::InvalidInput(
                        "Remote repository is not active".into(),
                    ));
                }
            }
        };

        let local_items = self.repo.get_all_items(user_id).await?;
        let mut processed_count: usize = 0;

        for item in local_items {
            // DIRECTIONAL SYNC: Skip items that are already synced
            // Only push items that are unsynced ('local_only' or 'modified')
            if item.sync_status == "synced" {
                continue; // Skip already-synced items
            }

            // Safety: never push an item under the wrong user scope.
            // This also guarantees the user_id sent to Supabase matches current session user_id.
            if item.user_id != user_id {
                warn!(
                    "Skipping push for item {} due to user_id mismatch (item.user_id={}, session.user_id={})",
                    item.id,
                    item.user_id,
                    user_id
                );
                continue;
            }

            let motivation = i8::try_from(item.motivation).map_err(
                |_| crate::error::AppError::InvalidInput(format!(
                    "Invalid motivation value for item {}: {}",
                    item.id, item.motivation
                ))
            )?;

            // Normalize timestamps to second-precision to avoid SQLite/Postgres mismatches
            let normalized_due = item.due.map(Self::normalize_timestamp);
            let _normalized_updated = Self::normalize_timestamp(item.updated_at);

            // Push item to remote: create or upsert
            remote_repo
                .create_item(
                    user_id,
                    item.id,
                    item.title.clone(),
                    motivation,
                    normalized_due,
                    item.duration_minutes,
                )
                .await?;

            // Push status using enum's as_str() method for safety
            remote_repo
                .update_item_status(user_id, item.id, item.status)
                .await?;

            // Push full item details
            remote_repo
                .update_item_details(
                    user_id,
                    item.id,
                    item.title,
                    item.description,
                    normalized_due,
                    item.duration_minutes,
                    motivation,
                )
                .await?;

            // Push soft-delete and archive state
            if item.deleted_at.is_some() {
                remote_repo.soft_delete_item(user_id, item.id).await?;
            } else {
                remote_repo.restore_item(user_id, item.id).await?;
                if item.is_archived {
                    remote_repo.archive_item(user_id, item.id).await?;
                } else {
                    remote_repo.unarchive_item(user_id, item.id).await?;
                }
            }

            // CRITICAL: Mark both sides as 'synced' after successful push
            // This ensures items are consistent and prevents re-sync on next cycle.
            
            // First, update remote DB (authoritative after push)
            if let Err(e) = remote_repo.update_sync_status(user_id, item.id, "synced").await {
                error!("Failed to mark remote item {} as synced: {:?}. Item will retry on next push.", item.id, e);
                continue; // Skip local update if remote fails
            }
            
            // Then update local DB. If this fails, remote is already 'synced'.
            // Next pull will sync the item back as 'synced', achieving eventual consistency.
            if let Err(e) = self.repo.update_sync_status(user_id, item.id, "synced").await {
                error!("Failed to mark local item {} as synced after remote push: {:?}. Item will sync on next pull.", item.id, e);
            } else {
                info!(
                    "Sync status updated to 'synced' for item {} in local SQLite after remote upload (user_id={})",
                    item.id,
                    user_id
                );
            }

            processed_count += 1;
        }

        let _ = self.app_handle.emit(
            "remote-catchup",
            RemoteCatchupEvent {
                user_id,
                synced_count: processed_count,
            },
        );

        Ok(processed_count)
    }

    // --- WRITE OPERATIONS ---

    pub async fn create_item(
        &self,
        user_id: Uuid,
        title: String,
        motivation: i8,
        due: Option<DateTime<Utc>>,
        duration_minutes: Option<i32>,
    ) -> AppResult<Uuid> {
        let id = Uuid::new_v4();

        // 1. Local Write (Immediate)
        self.repo.create_item(user_id, id, title.clone(), motivation, due, duration_minutes).await?;
        self.mark_local_sync_status(user_id, id, "local_only").await;
        info!("Local item created: {} ({}) for user {}", title, id, user_id);

        // 2. Remote Sync (Background)
        let remote_lock = self.remote.read().await;
        if let Some(ref remote_repo) = *remote_lock {
            let remote_repo = remote_repo.clone();
            let local_repo = self.repo.clone();
            let handle = self.app_handle.clone();
            let title_clone = title.clone();
            let user_id_copy = user_id;

            let _ = handle.emit("sync-status", SyncEvent { id, status: "pending".into(), message: None });

            tokio::spawn(async move {
                match remote_repo.create_item(user_id_copy, id, title_clone, motivation, due, duration_minutes).await {
                    Ok(_) => {
                        if let Err(e) = remote_repo.update_sync_status(user_id_copy, id, "synced").await {
                            error!("Failed to set remote sync status for {}: {}", id, e);
                        }
                        if let Err(e) = local_repo.update_sync_status(user_id_copy, id, "synced").await {
                            error!("Failed to set local sync status for {}: {}", id, e);
                        }
                        info!("Successfully synced item {} to Postgres", id);
                        let _ = handle.emit("sync-status", SyncEvent { id, status: "success".into(), message: None });
                    }
                    Err(e) => {
                        error!("Failed to sync item {}: {:?}", id, e);
                        let _ = handle.emit("sync-status", SyncEvent { id, status: "error".into(), message: Some(e.to_string()) });
                    }
                }
            });
        }
        Ok(id)
    }

    pub async fn update_item_status(&self, user_id: Uuid, id: Uuid, status: TaskStatus) -> AppResult<()> {
        self.repo.update_item_status(user_id, id, status.clone()).await?;
        self.mark_local_sync_status(user_id, id, "modified").await;
        
        let remote_lock = self.remote.read().await;
        if let Some(ref remote_repo) = *remote_lock {
            let remote_repo = remote_repo.clone();
            let local_repo = self.repo.clone();
            let handle = self.app_handle.clone();
            let status_clone = status.clone();
            let user_id_copy = user_id;

            let _ = handle.emit("sync-status", SyncEvent { id, status: "pending".into(), message: None });

            tokio::spawn(async move {
                if let Err(e) = remote_repo.update_item_status(user_id_copy, id, status_clone).await {
                    let _ = handle.emit("sync-status", SyncEvent { id, status: "error".into(), message: Some(e.to_string()) });
                } else {
                    if let Err(e) = remote_repo.update_sync_status(user_id_copy, id, "synced").await {
                        error!("Failed to set remote sync status for {}: {}", id, e);
                    }
                    if let Err(e) = local_repo.update_sync_status(user_id_copy, id, "synced").await {
                        error!("Failed to set local sync status for {}: {}", id, e);
                    }
                    let _ = handle.emit("sync-status", SyncEvent { id, status: "success".into(), message: None });
                }
            });
        }
        Ok(())
    }

    pub async fn update_item_details(
        &self,
        user_id: Uuid,
        id: Uuid,
        title: String,
        description: Option<String>,
        due: Option<DateTime<Utc>>,
        duration_minutes: Option<i32>,
        motivation: i8,
    ) -> AppResult<()> {
        self.repo.update_item_details(user_id, id, title.clone(), description.clone(), due, duration_minutes, motivation).await?;
        self.mark_local_sync_status(user_id, id, "modified").await;

        let remote_lock = self.remote.read().await;
        if let Some(ref remote_repo) = *remote_lock {
            let remote_repo = remote_repo.clone();
            let local_repo = self.repo.clone();
            let handle = self.app_handle.clone();
            let desc_clone = description.clone();
            let user_id_copy = user_id;
            
            let _ = handle.emit("sync-status", SyncEvent { id, status: "pending".into(), message: None });

            tokio::spawn(async move {
                if let Err(e) = remote_repo.update_item_details(user_id_copy, id, title, desc_clone, due, duration_minutes, motivation).await {
                    let _ = handle.emit("sync-status", SyncEvent { id, status: "error".into(), message: Some(e.to_string()) });
                } else {
                    if let Err(e) = remote_repo.update_sync_status(user_id_copy, id, "synced").await {
                        error!("Failed to set remote sync status for {}: {}", id, e);
                    }
                    if let Err(e) = local_repo.update_sync_status(user_id_copy, id, "synced").await {
                        error!("Failed to set local sync status for {}: {}", id, e);
                    }
                    let _ = handle.emit("sync-status", SyncEvent { id, status: "success".into(), message: None });
                }
            });
        }
        Ok(())
    }

    pub async fn archive_item(&self, user_id: Uuid, id: Uuid) -> AppResult<()> {
        self.repo.archive_item(user_id, id).await?;
        self.mark_local_sync_status(user_id, id, "modified").await;
        self.sync_simple_action(user_id, id, "archive").await;
        Ok(())
    }

    pub async fn unarchive_item(&self, user_id: Uuid, id: Uuid) -> AppResult<()> {
        self.repo.unarchive_item(user_id, id).await?;
        self.mark_local_sync_status(user_id, id, "modified").await;
        self.sync_simple_action(user_id, id, "unarchive").await;
        Ok(())
    }

    pub async fn soft_delete_item(&self, user_id: Uuid, id: Uuid) -> AppResult<()> {
        self.repo.soft_delete_item(user_id, id).await?;
        self.mark_local_sync_status(user_id, id, "modified").await;
        self.sync_simple_action(user_id, id, "soft-delete").await;
        Ok(())
    }

    pub async fn restore_item(&self, user_id: Uuid, id: Uuid) -> AppResult<()> {
        self.repo.restore_item(user_id, id).await?;
        self.mark_local_sync_status(user_id, id, "modified").await;
        self.sync_simple_action(user_id, id, "restore").await;
        Ok(())
    }

    pub async fn hard_delete_item(&self, user_id: Uuid, id: Uuid) -> AppResult<()> {
        self.repo.hard_delete_item(user_id, id).await?;
        self.sync_simple_action(user_id, id, "hard-delete").await;
        Ok(())
    }

    pub async fn empty_item_trash(&self, user_id: Uuid) -> AppResult<()> {
        self.repo.empty_item_trash(user_id, false).await?;
        let remote_lock = self.remote.read().await;
        if let Some(ref remote_repo) = *remote_lock {
            let remote_repo = remote_repo.clone();
            let user_id_copy = user_id;
            tokio::spawn(async move {
                if let Err(e) = remote_repo.empty_item_trash(user_id_copy, false).await {
                    error!("Failed to empty remote trash: {:?}", e);
                }else{
                    info!("Successfully emptied remote trash");
                }
            });
        }
        Ok(())
    }

    async fn sync_simple_action(&self, user_id: Uuid, id: Uuid, action_name: &'static str) {
        let remote_lock = self.remote.read().await;
        if let Some(ref remote_repo) = *remote_lock {
            let remote_repo = remote_repo.clone();
            let local_repo = self.repo.clone();
            let handle = self.app_handle.clone();
            let user_id_copy = user_id;
            
            let _ = handle.emit("sync-status", SyncEvent { id, status: "pending".into(), message: None });

            tokio::spawn(async move {
                let res = match action_name {
                    "archive" => remote_repo.archive_item(user_id_copy, id).await,
                    "unarchive" => remote_repo.unarchive_item(user_id_copy, id).await,
                    "soft-delete" => remote_repo.soft_delete_item(user_id_copy, id).await,
                    "restore" => remote_repo.restore_item(user_id_copy, id).await,
                    "hard-delete" => remote_repo.hard_delete_item(user_id_copy, id).await,
                    unknown => {
                        error!("Unknown sync action: {}", unknown);
                        Err(crate::error::AppError::InvalidInput(
                            format!("Unknown action: {}", unknown)
                        ))
                    }
                };

                match res {
                    Ok(_) => {
                        if action_name != "hard-delete" {
                            if let Err(e) = remote_repo.update_sync_status(user_id_copy, id, "synced").await {
                                error!("Failed to set remote sync status for {}: {}", id, e);
                            }
                            if let Err(e) = local_repo.update_sync_status(user_id_copy, id, "synced").await {
                                error!("Failed to set local sync status for {}: {}", id, e);
                            }
                        }
                        let _ = handle.emit("sync-status", SyncEvent { id, status: "success".into(), message: None });
                    }
                    Err(e) => {
                        error!("Sync failed for {} on {}: {}", action_name, id, e);
                        let _ = handle.emit("sync-status", SyncEvent { id, status: "error".into(), message: Some(e.to_string()) });
                    }
                }
            });
        }
    }
}