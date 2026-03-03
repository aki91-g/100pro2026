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

    pub async fn reset_all_databases(&self) -> AppResult<()> {
        self.local.empty_item_trash(true).await?;
        
        // Lock the remote for reading
        let remote_lock = self.remote.read().await;
        if let Some(ref remote_repo) = *remote_lock {
            remote_repo.empty_item_trash(true).await?;
        }
        Ok(())
    }

    pub async fn seed_test_data(&self) -> AppResult<()> {
        // 1. Define the data
        let seed_configs = vec![
            (Uuid::from_u128(0x00000000000000000000000000000001), "Backlog Item", "Planning stage", TaskStatus::Backlog, 0, false, false),
            (Uuid::from_u128(0x00000000000000000000000000000002), "InP Task", "Working on this", TaskStatus::InProgress, 5, false, false),
            (Uuid::from_u128(0x00000000000000000000000000000003), "Finished Task", "Ready to be archived", TaskStatus::Done, 2, false, false),
            (Uuid::from_u128(0x00000000000000000000000000000004), "Archived Project", "Past work", TaskStatus::InProgress, 0, true, false),
            (Uuid::from_u128(0x00000000000000000000000000000005), "Ghost Task", "This was deleted", TaskStatus::Todo, 0, false, false),
        ];
        let remote_lock = self.remote.read().await;
        
        for (id, title, desc, status, motivation, archived, deleted) in seed_configs {
            
            // Seed Local
            if let Err(e) = self.local.create_item(id, title.to_string(), motivation, None, None).await {
                eprintln!("❌ Failed to create local item '{}': {}", title, e);
                continue;
            }
            println!("✓ Created local item: {}", title);
            
            if let Err(e) = self.local.update_item_status(id, status).await {
                eprintln!("❌ Failed to set status for local item '{}': {}", title, e);
                continue;
            }
            
            if let Err(e) = self.local.update_item_details(id, title.to_string(), Some(desc.to_string()), None, None, motivation).await {
                eprintln!("❌ Failed to update local item details '{}': {}", title, e);
                continue;
            }
            
            if archived {
                if let Err(e) = self.local.archive_item(id).await {
                    eprintln!("❌ Failed to archive local item '{}': {}", title, e);
                }
            } else if let Err(e) = self.local.unarchive_item(id).await {
                eprintln!("❌ Failed to unarchive local item '{}': {}", title, e);
            }
            
            if deleted {
                if let Err(e) = self.local.soft_delete_item(id).await {
                    eprintln!("❌ Failed to soft-delete local item '{}': {}", title, e);
                }
            } else if let Err(e) = self.local.restore_item(id).await {
                eprintln!("❌ Failed to restore local item '{}': {}", title, e);
            }
            
            // Seed Remote
            if let Some(ref remote_repo) = *remote_lock {
                if let Err(e) = remote_repo.create_item(id, title.to_string(), motivation, None, None).await {
                    eprintln!("❌ Failed to create remote item '{}': {}", title, e);
                    continue;
                }
                println!("✓ Pushed to Supabase: {}", title);
                
                if let Err(e) = remote_repo.update_item_status(id, status).await {
                    eprintln!("❌ Failed to set status for remote item '{}': {}", title, e);
                }
                
                if let Err(e) = remote_repo.update_item_details(id, title.to_string(), Some(desc.to_string()), None, None, motivation).await {
                    eprintln!("❌ Failed to update remote item details '{}': {}", title, e);
                }
                
                if archived {
                    if let Err(e) = remote_repo.archive_item(id).await {
                        eprintln!("❌ Failed to archive remote item '{}': {}", title, e);
                    }
                } else if let Err(e) = remote_repo.unarchive_item(id).await {
                    eprintln!("❌ Failed to unarchive remote item '{}': {}", title, e);
                }
                
                if deleted {
                    if let Err(e) = remote_repo.soft_delete_item(id).await {
                        eprintln!("❌ Failed to soft-delete remote item '{}': {}", title, e);
                    }
                } else if let Err(e) = remote_repo.restore_item(id).await {
                    eprintln!("❌ Failed to restore remote item '{}': {}", title, e);
                }
            }
        }
        Ok(())
    }
}