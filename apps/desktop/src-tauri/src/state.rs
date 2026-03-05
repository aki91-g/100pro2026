use tokio::sync::RwLock;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    current_user_id: Arc<RwLock<Option<Uuid>>>,
    current_username: Arc<RwLock<Option<String>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            current_user_id: Arc::new(RwLock::new(None)),
            current_username: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a new AppState with a specific user ID
    pub fn with_user(user_id: Uuid) -> Self {
        Self {
            current_user_id: Arc::new(RwLock::new(Some(user_id))),
            current_username: Arc::new(RwLock::new(None)),
        }
    }

    /// Get the current user ID
    pub async fn get_user_id(&self) -> Result<Uuid, String> {
        let user_id = self.current_user_id.read().await;
        user_id
            .clone()
            .ok_or_else(|| "No user logged in".to_string())
    }

    /// Get the current username
    pub async fn get_username(&self) -> Option<String> {
        let username = self.current_username.read().await;
        username.clone()
    }

    /// Set the current user ID and username (called during login)
    pub async fn set_user_id(&self, user_id: Uuid) {
        let mut current = self.current_user_id.write().await;
        *current = Some(user_id);
    }

    /// Set the current username
    pub async fn set_username(&self, username: String) {
        let mut current = self.current_username.write().await;
        *current = Some(username);
    }

    /// Set both user ID and username
    pub async fn set_user(&self, user_id: Uuid, username: String) {
        self.set_user_id(user_id).await;
        self.set_username(username).await;
    }

    /// Clear the current user ID and username (called during logout)
    pub async fn clear_user_id(&self) {
        let mut current_id = self.current_user_id.write().await;
        let mut current_name = self.current_username.write().await;
        *current_id = None;
        *current_name = None;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
