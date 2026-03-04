use async_trait::async_trait;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::item::{Item, TaskStatus};
use crate::error::AppResult;

#[async_trait]
pub trait ItemRepository: Send + Sync {
    // GET logic - all scoped by user_id
    async fn get_all_items(&self, user_id: &str) -> AppResult<Vec<Item>>;
    async fn get_active_items(&self, user_id: &str) -> AppResult<Vec<Item>>;
    async fn get_archived_items(&self, user_id: &str) -> AppResult<Vec<Item>>;
    async fn get_deleted_items(&self, user_id: &str) -> AppResult<Vec<Item>>;

    // POST logic
    async fn create_item(&self, user_id: &str, id: Uuid, title: String, motivation: i8, due: Option<DateTime<Utc>>, duration_minutes: Option<i32>) -> AppResult<()>;

    // UPDATE logic
    async fn update_item_status(&self, user_id: &str, id: Uuid, status: TaskStatus) -> AppResult<()>;
    async fn update_item_details(&self, user_id: &str, id: Uuid, title: String, description: Option<String>, due: Option<DateTime<Utc>>, duration_minutes: Option<i32>, motivation: i8) -> AppResult<()>;

    // ARCHIVE logic
    async fn archive_item(&self, user_id: &str, id: Uuid) -> AppResult<()>;
    async fn unarchive_item(&self, user_id: &str, id: Uuid) -> AppResult<()>;

    // SOFT DELETE logic
    async fn soft_delete_item(&self, user_id: &str, id: Uuid) -> AppResult<()>;
    async fn restore_item(&self, user_id: &str, id: Uuid) -> AppResult<()>;

    // HARD DELETE logic
    async fn hard_delete_item(&self, user_id: &str, id: Uuid) -> AppResult<()>;
    async fn empty_item_trash(&self, user_id: &str, full_wipe: bool) -> AppResult<()>;

    // CLAIM OFFLINE ITEMS - for post-login claiming
    async fn claim_offline_items(&self, user_id: &str) -> AppResult<usize>;
}