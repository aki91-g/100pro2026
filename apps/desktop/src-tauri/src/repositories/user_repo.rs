use async_trait::async_trait;
use crate::error::AppResult;
use crate::models::user::{LocalUser, LocalUserUpdate};

/// Repository trait for managing local users  
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Get the active local user (if any)
    async fn get_active_user(&self) -> AppResult<Option<LocalUser>>;
    
    /// Get a user by ID
    async fn get_user_by_id(&self, user_id: &str) -> AppResult<Option<LocalUser>>;
    
    /// Create or update a local user
    async fn upsert_user(&self, user: &LocalUserUpdate) -> AppResult<()>;
    
    /// Update the last_login timestamp for a user
    async fn update_last_login(&self, user_id: &str) -> AppResult<()>;
    
    /// Deactivate a user (logout)
    async fn deactivate_user(&self, user_id: &str) -> AppResult<()>;
    
    /// Clear all users (for testing/debug)
    async fn clear_all_users(&self) -> AppResult<()>;
}
