use std::{f32::consts::E, sync::Arc};
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
    pub async fn get_active_items(&self) -> AppResult<Vec<Item>> {
        self.repo.get_active_items().await
    }

    pub async fn get_archived_items(&self) -> AppResult<Vec<Item>> {
        self.repo.get_archived_items().await
    }

    pub async fn get_deleted_items(&self) -> AppResult<Vec<Item>> {
        self.repo.get_deleted_items().await
    }

    pub async fn sync_items(&self) -> AppResult<usize> {
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

        let remote_items = remote_repo.get_all_items().await?;
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
                    item.id,
                    item.title.clone(),
                    motivation,
                    item.due,
                    item.duration_minutes,
                )
                .await?;

            self.repo.update_item_status(item.id, item.status).await?;

            self.repo
                .update_item_details(
                    item.id,
                    item.title,
                    item.description,
                    item.due,
                    item.duration_minutes,
                    motivation,
                )
                .await?;

            if item.is_archived {
                self.repo.archive_item(item.id).await?;
            } else {
                self.repo.unarchive_item(item.id).await?;
            }

            if item.deleted_at.is_some() {
                self.repo.soft_delete_item(item.id).await?;
            } else {
                self.repo.restore_item(item.id).await?;
            }

            processed_count += 1;
        }

        // Note: Local-only items are preserved as they may be pending sync (created while offline or not yet synced). They will naturally sync on the next successful remote connection.

        Ok(processed_count)
    }

    // --- WRITE OPERATIONS ---

    pub async fn create_item(
        &self,
        title: String,
        motivation: i8,
        due: Option<DateTime<Utc>>,
        duration_minutes: Option<i32>,
    ) -> AppResult<Uuid> {
        let id = Uuid::new_v4();

        // 1. Local Write (Immediate)
        self.repo.create_item(id, title.clone(), motivation, due, duration_minutes).await?;
        info!("Local item created: {} ({})", title, id);

        // 2. Remote Sync (Background)
        let remote_lock = self.remote.read().await;
        if let Some(ref remote_repo) = *remote_lock {
            let remote_repo = remote_repo.clone();
            let handle = self.app_handle.clone();
            let title_clone = title.clone();

            let _ = handle.emit("sync-status", SyncEvent { id, status: "pending".into(), message: None });

            tokio::spawn(async move {
                match remote_repo.create_item(id, title_clone, motivation, due, duration_minutes).await {
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

    pub async fn update_item_status(&self, id: Uuid, status: TaskStatus) -> AppResult<()> {
        self.repo.update_item_status(id, status.clone()).await?;
        
        let remote_lock = self.remote.read().await;
        if let Some(ref remote_repo) = *remote_lock {
            let remote_repo = remote_repo.clone();
            let handle = self.app_handle.clone();
            let status_clone = status.clone();

            let _ = handle.emit("sync-status", SyncEvent { id, status: "pending".into(), message: None });

            tokio::spawn(async move {
                if let Err(e) = remote_repo.update_item_status(id, status_clone).await {
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
        id: Uuid,
        title: String,
        description: Option<String>,
        due: Option<DateTime<Utc>>,
        duration_minutes: Option<i32>,
        motivation: i8,
    ) -> AppResult<()> {
        self.repo.update_item_details(id, title.clone(), description.clone(), due, duration_minutes, motivation).await?;

        let remote_lock = self.remote.read().await;
        if let Some(ref remote_repo) = *remote_lock {
            let remote_repo = remote_repo.clone();
            let handle = self.app_handle.clone();
            let desc_clone = description.clone();
            
            let _ = handle.emit("sync-status", SyncEvent { id, status: "pending".into(), message: None });

            tokio::spawn(async move {
                if let Err(e) = remote_repo.update_item_details(id, title, desc_clone, due, duration_minutes, motivation).await {
                    let _ = handle.emit("sync-status", SyncEvent { id, status: "error".into(), message: Some(e.to_string()) });
                } else {
                    let _ = handle.emit("sync-status", SyncEvent { id, status: "success".into(), message: None });
                }
            });
        }
        Ok(())
    }

    pub async fn archive_item(&self, id: Uuid) -> AppResult<()> {
        self.repo.archive_item(id).await?;
        self.sync_simple_action(id, "archive").await;
        Ok(())
    }

    pub async fn unarchive_item(&self, id: Uuid) -> AppResult<()> {
        self.repo.unarchive_item(id).await?;
        self.sync_simple_action(id, "unarchive").await;
        Ok(())
    }

    pub async fn soft_delete_item(&self, id: Uuid) -> AppResult<()> {
        self.repo.soft_delete_item(id).await?;
        self.sync_simple_action(id, "soft-delete").await;
        Ok(())
    }

    pub async fn restore_item(&self, id: Uuid) -> AppResult<()> {
        self.repo.restore_item(id).await?;
        self.sync_simple_action(id, "restore").await;
        Ok(())
    }

    pub async fn hard_delete_item(&self, id: Uuid) -> AppResult<()> {
        self.repo.hard_delete_item(id).await?;
        self.sync_simple_action(id, "hard-delete").await;
        Ok(())
    }

    pub async fn empty_item_trash(&self) -> AppResult<()> {
        self.repo.empty_item_trash(false).await?;
        let remote_lock = self.remote.read().await;
        if let Some(ref remote_repo) = *remote_lock {
            let remote_repo = remote_repo.clone();
            tokio::spawn(async move {
                if let Err(e) = remote_repo.empty_item_trash(false).await {
                    error!("Failed to empty remote trash: {:?}", e);
                }else{
                    info!("Successfully emptied remote trash");
                }
            });
        }
        Ok(())
    }

    async fn sync_simple_action(&self, id: Uuid, action_name: &'static str) {
        let remote_lock = self.remote.read().await;
        if let Some(ref remote_repo) = *remote_lock {
            let remote_repo = remote_repo.clone();
            let handle = self.app_handle.clone();
            
            let _ = handle.emit("sync-status", SyncEvent { id, status: "pending".into(), message: None });

            tokio::spawn(async move {
                let res = match action_name {
                    "archive" => remote_repo.archive_item(id).await,
                    "unarchive" => remote_repo.unarchive_item(id).await,
                    "soft-delete" => remote_repo.soft_delete_item(id).await,
                    "restore" => remote_repo.restore_item(id).await,
                    "hard-delete" => remote_repo.hard_delete_item(id).await,
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