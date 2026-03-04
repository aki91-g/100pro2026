use tokio::sync::RwLock;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    current_user_id: Arc<RwLock<Option<String>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            current_user_id: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a new AppState with a specific user ID
    pub fn with_user(user_id: String) -> Self {
        Self {
            current_user_id: Arc::new(RwLock::new(Some(user_id))),
        }
    }

    /// Get the current user ID
    pub async fn get_user_id(&self) -> Result<String, String> {
        let user_id = self.current_user_id.read().await;
        user_id
            .clone()
            .ok_or_else(|| "No user logged in".to_string())
    }

    /// Set the current user ID (called during login)
    pub async fn set_user_id(&self, user_id: String) {
        let mut current = self.current_user_id.write().await;
        *current = Some(user_id);
    }

    /// Clear the current user ID (called during logout)
    pub async fn clear_user_id(&self) {
        let mut current = self.current_user_id.write().await;
        *current = None;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
