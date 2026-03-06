use async_trait::async_trait;
use crate::error::AppResult;
use crate::models::session::LocalSession;

/// Repository trait for managing local session
#[async_trait]
pub trait SessionRepository: Send + Sync {
    /// Get the active session (if any)
    async fn get_active_session(&self) -> AppResult<Option<LocalSession>>;
    
    /// Save/Update the active session (only one can exist)
    async fn save_session(&self, user_id: &uuid::Uuid, username: &str, access_token: Option<&str>) -> AppResult<()>;
    
    /// Clear the active session (logout)
    async fn clear_session(&self) -> AppResult<()>;
    
    /// Update last_login timestamp for the active session
    async fn update_last_login(&self) -> AppResult<()>;
}
