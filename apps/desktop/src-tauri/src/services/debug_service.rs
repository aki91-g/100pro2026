use std::sync::Arc;
use crate::error::AppResult;
use crate::models::item::TaskStatus;
use crate::repositories::item_repo::ItemRepository;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct DebugService {
    local: Arc<dyn ItemRepository>,
    remote: RwLock<Option<Arc<dyn ItemRepository>>>,
}

impl DebugService {
    pub fn new(local: Arc<dyn ItemRepository>, remote: Option<Arc<dyn ItemRepository>>) -> Self {
        Self { 
            local, 
            remote: RwLock::new(remote) 
        }
    }

    // New method to activate remote debugging once Supabase connects
    pub async fn set_remote(&self, remote_repo: Arc<dyn ItemRepository>) {
        let mut w = self.remote.write().await;
        *w = Some(remote_repo);
    }

    pub async fn has_remote(&self) -> bool {
        self.remote.read().await.is_some()
    }

    pub async fn reset_all_databases(&self, user_id: Uuid) -> AppResult<()> {
        // 1. Hard-delete all items for the user (including archived/deleted)
        let all_items = self.local.get_all_items(user_id).await?;
        for item in all_items {
            // Hard-delete each item to remove all traces
            self.local.hard_delete_item(user_id, item.id).await?;
        }
        
        // 2. Clear remote as well if available
        let remote_repo = {
            let remote_lock = self.remote.read().await;
            remote_lock.clone()
        };
        if let Some(remote_repo) = remote_repo {
            // Get all remote items and hard-delete them
            let remote_items = remote_repo.get_all_items(user_id).await?;
            for item in remote_items {
                remote_repo.hard_delete_item(user_id, item.id).await?;
            }
        }
        
        // 3. Final cleanup: empty any remaining trash
        self.local.empty_item_trash(user_id, true).await?;
        if let Some(remote_repo) = {
            let remote_lock = self.remote.read().await;
            remote_lock.clone()
        } {
            remote_repo.empty_item_trash(user_id, true).await?;
        }
        
        Ok(())
    }

    pub async fn seed_test_data(&self, user_id: Uuid) -> AppResult<()> {
        // 1. Define the seed data with hardcoded UUIDs
        let seed_configs = vec![
            (Uuid::from_u128(0x00000000000000000000000000000001), "Backlog Item", "Planning stage", TaskStatus::Backlog, 0, false, false),
            (Uuid::from_u128(0x00000000000000000000000000000002), "InP Task", "Working on this", TaskStatus::InProgress, 5, false, false),
            (Uuid::from_u128(0x00000000000000000000000000000003), "Finished Task", "Ready to be archived", TaskStatus::Done, 2, false, false),
            (Uuid::from_u128(0x00000000000000000000000000000004), "Archived Project", "Past work", TaskStatus::InProgress, 0, true, false),
            (Uuid::from_u128(0x00000000000000000000000000000005), "Ghost Task", "This was deleted", TaskStatus::Todo, 0, false, true),
        ];

        // 2. Check if any seed UUID already exists (including archived/deleted items)
        // Try to create items and handle duplicate-key errors gracefully
        let remote_lock = self.remote.read().await;
        
        for (id, title, desc, status, motivation, archived, deleted) in seed_configs {
            // Attempt to create item; if it already exists, skip it gracefully
            match self.local.create_item(user_id, id, title.to_string(), motivation, None, None).await {
                Ok(_) => {
                    println!("✓ Created local item: {}", title);
                },
                Err(e) if e.to_string().contains("UNIQUE constraint failed") || 
                         e.to_string().contains("duplicate") => {
                    // Item with this UUID already exists (may be archived/deleted); skip it
                    println!("⊘ Item '{}' already exists (skipping)", title);
                    continue;
                },
                Err(e) => return Err(e), // Other errors should fail
            }
            
            self.local.update_item_status(user_id, id, status).await?;
            
            self.local
                .update_item_details(user_id, id, title.to_string(), Some(desc.to_string()), None, None, motivation)
                .await?;
            
            if archived {
                self.local.archive_item(user_id, id).await?;
            } else {
                self.local.unarchive_item(user_id, id).await?;
            }
            
            if deleted {
                self.local.soft_delete_item(user_id, id).await?;
            } else {
                self.local.restore_item(user_id, id).await?;
            }
            
            // Seed Remote
            if let Some(ref remote_repo) = *remote_lock {
                // Attempt remote create with same duplicate-key handling
                match remote_repo.create_item(user_id, id, title.to_string(), motivation, None, None).await {
                    Ok(_) => {
                        println!("✓ Pushed to Supabase: {}", title);
                    },
                    Err(e) if e.to_string().contains("UNIQUE constraint failed") || 
                             e.to_string().contains("duplicate") => {
                        println!("⊘ Remote item '{}' already exists (skipping remote seed)", title);
                        continue;
                    },
                    Err(e) => return Err(e),
                }
                
                remote_repo.update_item_status(user_id, id, status).await?;
                
                remote_repo
                    .update_item_details(user_id, id, title.to_string(), Some(desc.to_string()), None, None, motivation)
                    .await?;
                
                if archived {
                    remote_repo.archive_item(user_id, id).await?;
                } else {
                    remote_repo.unarchive_item(user_id, id).await?;
                }
                
                if deleted {
                    remote_repo.soft_delete_item(user_id, id).await?;
                } else {
                    remote_repo.restore_item(user_id, id).await?;
                }
            }
        }
        Ok(())
    }

}