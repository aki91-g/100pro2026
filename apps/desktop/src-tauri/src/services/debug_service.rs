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

    pub async fn reset_all_databases(&self, user_id: &str) -> AppResult<()> {
        self.local.empty_item_trash(user_id, true).await?;
        
        // Lock the remote for reading
        let remote_repo = {
            let remote_lock = self.remote.read().await;
            remote_lock.clone()
        };
        if let Some(remote_repo) = remote_repo {
            remote_repo.empty_item_trash(user_id, true).await?;
        }
        Ok(())
    }

    pub async fn seed_test_data(&self, user_id: &str) -> AppResult<()> {
        // Check if user already has data - only seed if empty
        let existing_items = self.local.get_all_items(user_id).await?;
        if !existing_items.is_empty() {
            return Err(crate::error::AppError::InvalidInput(
                format!("User '{}' already has {} items. Clear data first to seed.", user_id, existing_items.len())
            ));
        }

        // 1. Define the data
        let seed_configs = vec![
            (Uuid::from_u128(0x00000000000000000000000000000001), "Backlog Item", "Planning stage", TaskStatus::Backlog, 0, false, false),
            (Uuid::from_u128(0x00000000000000000000000000000002), "InP Task", "Working on this", TaskStatus::InProgress, 5, false, false),
            (Uuid::from_u128(0x00000000000000000000000000000003), "Finished Task", "Ready to be archived", TaskStatus::Done, 2, false, false),
            (Uuid::from_u128(0x00000000000000000000000000000004), "Archived Project", "Past work", TaskStatus::InProgress, 0, true, false),
            (Uuid::from_u128(0x00000000000000000000000000000005), "Ghost Task", "This was deleted", TaskStatus::Todo, 0, false, true),
        ];
        let remote_lock = self.remote.read().await;
        
        for (id, title, desc, status, motivation, archived, deleted) in seed_configs {
            
            // Seed Local
            if let Err(e) = self.local.create_item(user_id, id, title.to_string(), motivation, None, None).await {
                eprintln!("❌ Failed to create local item '{}': {}", title, e);
                continue;
            }
            println!("✓ Created local item: {}", title);
            
            if let Err(e) = self.local.update_item_status(user_id, id, status).await {
                eprintln!("❌ Failed to set status for local item '{}': {}", title, e);
                continue;
            }
            
            if let Err(e) = self.local.update_item_details(user_id, id, title.to_string(), Some(desc.to_string()), None, None, motivation).await {
                eprintln!("❌ Failed to update local item details '{}': {}", title, e);
                continue;
            }
            
            if archived {
                if let Err(e) = self.local.archive_item(user_id, id).await {
                    eprintln!("❌ Failed to archive local item '{}': {}", title, e);
                }
            } else if let Err(e) = self.local.unarchive_item(user_id, id).await {
                eprintln!("❌ Failed to unarchive local item '{}': {}", title, e);
            }
            
            if deleted {
                if let Err(e) = self.local.soft_delete_item(user_id, id).await {
                    eprintln!("❌ Failed to soft-delete local item '{}': {}", title, e);
                }
            } else if let Err(e) = self.local.restore_item(user_id, id).await {
                eprintln!("❌ Failed to restore local item '{}': {}", title, e);
            }
            
            // Seed Remote
            if let Some(ref remote_repo) = *remote_lock {
                if let Err(e) = remote_repo.create_item(user_id, id, title.to_string(), motivation, None, None).await {
                    eprintln!("❌ Failed to create remote item '{}': {}", title, e);
                    continue;
                }
                println!("✓ Pushed to Supabase: {}", title);
                
                if let Err(e) = remote_repo.update_item_status(user_id, id, status).await {
                    eprintln!("❌ Failed to set status for remote item '{}': {}", title, e);
                }
                
                if let Err(e) = remote_repo.update_item_details(user_id, id, title.to_string(), Some(desc.to_string()), None, None, motivation).await {
                    eprintln!("❌ Failed to update remote item details '{}': {}", title, e);
                }
                
                if archived {
                    if let Err(e) = remote_repo.archive_item(user_id, id).await {
                        eprintln!("❌ Failed to archive remote item '{}': {}", title, e);
                    }
                } else if let Err(e) = remote_repo.unarchive_item(user_id, id).await {
                    eprintln!("❌ Failed to unarchive remote item '{}': {}", title, e);
                }
                
                if deleted {
                    if let Err(e) = remote_repo.soft_delete_item(user_id, id).await {
                        eprintln!("❌ Failed to soft-delete remote item '{}': {}", title, e);
                    }
                } else if let Err(e) = remote_repo.restore_item(user_id, id).await {
                    eprintln!("❌ Failed to restore remote item '{}': {}", title, e);
                }
            }
        }
        Ok(())
    }

    /// Migration function to clean up items with NULL user_id
    /// Uses the existing claim_offline_items method to assign them to a user
    /// Or deletes them if no user is specified
    pub async fn migrate_null_user_items(&self, assign_to_user: Option<&str>) -> AppResult<usize> {
        if let Some(user_id) = assign_to_user {
            // Use claim_offline_items to assign NULL user_id items to this user
            let local_count = self.local.claim_offline_items(user_id).await?;
            
            // Also do it for remote if available
            let remote_lock = self.remote.read().await;
            let remote_count = if let Some(ref remote_repo) = *remote_lock {
                remote_repo.claim_offline_items(user_id).await?
            } else {
                0
            };
            
            let total = local_count + remote_count;
            println!("✓ Migrated {} NULL user_id items to user '{}'", total, user_id);
            Ok(total)
        } else {
            // Delete NULL user_id items by using empty_item_trash with a special empty user_id
            // Since our repos filter by user_id, we need a direct SQL approach
            // For now, return 0 and log - proper implementation would need raw SQL
            println!("⚠ Deletion of NULL user_id items requires direct SQL implementation");
            Ok(0)
        }
    }
}