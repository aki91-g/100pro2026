use std::{sync::Arc};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tauri::{AppHandle, Emitter};
use tracing::{info, error};
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
    user_id: String,
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

    // --- READ OPERATIONS ---
    pub async fn get_active_items(&self, user_id: &str) -> AppResult<Vec<Item>> {
        self.repo.get_active_items(user_id).await
    }

    pub async fn get_archived_items(&self, user_id: &str) -> AppResult<Vec<Item>> {
        self.repo.get_archived_items(user_id).await
    }

    pub async fn get_deleted_items(&self, user_id: &str) -> AppResult<Vec<Item>> {
        self.repo.get_deleted_items(user_id).await
    }

    // --- CLAIM OFFLINE ITEMS ---
    pub async fn claim_offline_items(&self, user_id: &str) -> AppResult<usize> {
        // Claim items in local repo
        let local_count = self.repo.claim_offline_items(user_id).await?;
        info!("Claimed {} offline items locally for user {}", local_count, user_id);

        // Also claim on remote if available
        let remote_lock = self.remote.read().await;
        if let Some(ref remote_repo) = *remote_lock {
            let remote_repo = remote_repo.clone();
            let user_id_clone = user_id.to_string();
            
            // Spawn background task for remote claiming
            tokio::spawn(async move {
                match remote_repo.claim_offline_items(&user_id_clone).await {
                    Ok(count) => {
                        info!("Claimed {} offline items remotely for user {}", count, user_id_clone);
                    }
                    Err(e) => {
                        error!("Failed to claim offline items remotely: {:?}", e);
                    }
                }
            });
        }

        Ok(local_count)
    }

    pub async fn sync_items(&self, user_id: &str) -> AppResult<usize> {
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
        let mut processed_count: usize = 0;

        for item in remote_items {
            let motivation = i8::try_from(item.motivation).map_err(
                |_| crate::error::AppError::InvalidInput(format!(
                    "Invalid motivation value for item {}: {}", 
                    item.id, item.motivation
                ))
            )?;
            self.repo
                .create_item(
                    user_id,
                    item.id,
                    item.title.clone(),
                    motivation,
                    item.due,
                    item.duration_minutes,
                )
                .await?;

            self.repo.update_item_status(user_id, item.id, item.status).await?;

            // Always call update_item_details to sync description
            // (pass None explicitly to clear remote-deleted descriptions)
            self.repo
                .update_item_details(
                    user_id,
                    item.id,
                    item.title,
                    item.description,
                    item.due,
                    item.duration_minutes,
                    motivation,
                )
                .await?;

            // Only call archive_item if remote is archived
            // (newly created items are unarchived by default)
            if item.is_archived {
                self.repo.archive_item(user_id, item.id).await?;
            }

            // Only call soft_delete_item if remote is deleted
            // (newly created items are not deleted by default)
            if item.deleted_at.is_some() {
                self.repo.soft_delete_item(user_id, item.id).await?;
            }

            processed_count += 1;
        }

        // Note: Local-only items are preserved as they may be pending sync (created while offline or not yet synced). They will naturally sync on the next successful remote connection.

        Ok(processed_count)
    }

    pub async fn sync_local_to_remote(&self, user_id: &str) -> AppResult<usize> {
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
            let motivation = i8::try_from(item.motivation).map_err(
                |_| crate::error::AppError::InvalidInput(format!(
                    "Invalid motivation value for item {}: {}",
                    item.id, item.motivation
                ))
            )?;

            remote_repo
                .create_item(
                    user_id,
                    item.id,
                    item.title.clone(),
                    motivation,
                    item.due,
                    item.duration_minutes,
                )
                .await?;

            remote_repo
                .update_item_status(user_id, item.id, item.status)
                .await?;

            remote_repo
                .update_item_details(
                    user_id,
                    item.id,
                    item.title,
                    item.description,
                    item.due,
                    item.duration_minutes,
                    motivation,
                )
                .await?;

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

            processed_count += 1;
        }

        let _ = self.app_handle.emit(
            "remote-catchup",
            RemoteCatchupEvent {
                user_id: user_id.to_string(),
                synced_count: processed_count,
            },
        );

        Ok(processed_count)
    }

    // --- WRITE OPERATIONS ---

    pub async fn create_item(
        &self,
        user_id: &str,
        title: String,
        motivation: i8,
        due: Option<DateTime<Utc>>,
        duration_minutes: Option<i32>,
    ) -> AppResult<Uuid> {
        let id = Uuid::new_v4();

        // 1. Local Write (Immediate)
        self.repo.create_item(user_id, id, title.clone(), motivation, due, duration_minutes).await?;
        info!("Local item created: {} ({}) for user {}", title, id, user_id);

        // 2. Remote Sync (Background)
        let remote_lock = self.remote.read().await;
        if let Some(ref remote_repo) = *remote_lock {
            let remote_repo = remote_repo.clone();
            let handle = self.app_handle.clone();
            let title_clone = title.clone();
            let user_id_clone = user_id.to_string();

            let _ = handle.emit("sync-status", SyncEvent { id, status: "pending".into(), message: None });

            tokio::spawn(async move {
                match remote_repo.create_item(&user_id_clone, id, title_clone, motivation, due, duration_minutes).await {
                    Ok(_) => {
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

    pub async fn update_item_status(&self, user_id: &str, id: Uuid, status: TaskStatus) -> AppResult<()> {
        self.repo.update_item_status(user_id, id, status.clone()).await?;
        
        let remote_lock = self.remote.read().await;
        if let Some(ref remote_repo) = *remote_lock {
            let remote_repo = remote_repo.clone();
            let handle = self.app_handle.clone();
            let status_clone = status.clone();
            let user_id_clone = user_id.to_string();

            let _ = handle.emit("sync-status", SyncEvent { id, status: "pending".into(), message: None });

            tokio::spawn(async move {
                if let Err(e) = remote_repo.update_item_status(&user_id_clone, id, status_clone).await {
                    let _ = handle.emit("sync-status", SyncEvent { id, status: "error".into(), message: Some(e.to_string()) });
                } else {
                    let _ = handle.emit("sync-status", SyncEvent { id, status: "success".into(), message: None });
                }
            });
        }
        Ok(())
    }

    pub async fn update_item_details(
        &self,
        user_id: &str,
        id: Uuid,
        title: String,
        description: Option<String>,
        due: Option<DateTime<Utc>>,
        duration_minutes: Option<i32>,
        motivation: i8,
    ) -> AppResult<()> {
        self.repo.update_item_details(user_id, id, title.clone(), description.clone(), due, duration_minutes, motivation).await?;

        let remote_lock = self.remote.read().await;
        if let Some(ref remote_repo) = *remote_lock {
            let remote_repo = remote_repo.clone();
            let handle = self.app_handle.clone();
            let desc_clone = description.clone();
            let user_id_clone = user_id.to_string();
            
            let _ = handle.emit("sync-status", SyncEvent { id, status: "pending".into(), message: None });

            tokio::spawn(async move {
                if let Err(e) = remote_repo.update_item_details(&user_id_clone, id, title, desc_clone, due, duration_minutes, motivation).await {
                    let _ = handle.emit("sync-status", SyncEvent { id, status: "error".into(), message: Some(e.to_string()) });
                } else {
                    let _ = handle.emit("sync-status", SyncEvent { id, status: "success".into(), message: None });
                }
            });
        }
        Ok(())
    }

    pub async fn archive_item(&self, user_id: &str, id: Uuid) -> AppResult<()> {
        self.repo.archive_item(user_id, id).await?;
        self.sync_simple_action(user_id, id, "archive").await;
        Ok(())
    }

    pub async fn unarchive_item(&self, user_id: &str, id: Uuid) -> AppResult<()> {
        self.repo.unarchive_item(user_id, id).await?;
        self.sync_simple_action(user_id, id, "unarchive").await;
        Ok(())
    }

    pub async fn soft_delete_item(&self, user_id: &str, id: Uuid) -> AppResult<()> {
        self.repo.soft_delete_item(user_id, id).await?;
        self.sync_simple_action(user_id, id, "soft-delete").await;
        Ok(())
    }

    pub async fn restore_item(&self, user_id: &str, id: Uuid) -> AppResult<()> {
        self.repo.restore_item(user_id, id).await?;
        self.sync_simple_action(user_id, id, "restore").await;
        Ok(())
    }

    pub async fn hard_delete_item(&self, user_id: &str, id: Uuid) -> AppResult<()> {
        self.repo.hard_delete_item(user_id, id).await?;
        self.sync_simple_action(user_id, id, "hard-delete").await;
        Ok(())
    }

    pub async fn empty_item_trash(&self, user_id: &str) -> AppResult<()> {
        self.repo.empty_item_trash(user_id, false).await?;
        let remote_lock = self.remote.read().await;
        if let Some(ref remote_repo) = *remote_lock {
            let remote_repo = remote_repo.clone();
            let user_id_clone = user_id.to_string();
            tokio::spawn(async move {
                if let Err(e) = remote_repo.empty_item_trash(&user_id_clone, false).await {
                    error!("Failed to empty remote trash: {:?}", e);
                }else{
                    info!("Successfully emptied remote trash");
                }
            });
        }
        Ok(())
    }

    async fn sync_simple_action(&self, user_id: &str, id: Uuid, action_name: &'static str) {
        let remote_lock = self.remote.read().await;
        if let Some(ref remote_repo) = *remote_lock {
            let remote_repo = remote_repo.clone();
            let handle = self.app_handle.clone();
            let user_id_clone = user_id.to_string();
            
            let _ = handle.emit("sync-status", SyncEvent { id, status: "pending".into(), message: None });

            tokio::spawn(async move {
                let res = match action_name {
                    "archive" => remote_repo.archive_item(&user_id_clone, id).await,
                    "unarchive" => remote_repo.unarchive_item(&user_id_clone, id).await,
                    "soft-delete" => remote_repo.soft_delete_item(&user_id_clone, id).await,
                    "restore" => remote_repo.restore_item(&user_id_clone, id).await,
                    "hard-delete" => remote_repo.hard_delete_item(&user_id_clone, id).await,
                    unknown => {
                        error!("Unknown sync action: {}", unknown);
                        Err(crate::error::AppError::InvalidInput(
                            format!("Unknown action: {}", unknown)
                        ))
                    }
                };

                match res {
                    Ok(_) => {
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