use async_trait::async_trait;
use crate::error::AppResult;
use crate::models::user::Profile;

/// Repository trait for managing profiles in Postgres
#[async_trait]
pub trait ProfileRepository: Send + Sync {
    /// Get a profile by user ID
    async fn get_profile(&self, user_id: &str) -> AppResult<Option<Profile>>;
    
    /// Create or update a profile
    async fn upsert_profile(&self, user_id: &str, username: &str) -> AppResult<()>;
    
    /// Delete a profile
    async fn delete_profile(&self, user_id: &str) -> AppResult<()>;
}
