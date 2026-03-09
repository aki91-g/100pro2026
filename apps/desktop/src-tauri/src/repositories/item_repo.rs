use async_trait::async_trait;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::item::{Item, TaskStatus};
use crate::error::AppResult;

#[async_trait]
pub trait ItemRepository: Send + Sync {
    // GET logic - all scoped by user_id
    async fn get_all_items(&self, user_id: Uuid) -> AppResult<Vec<Item>>;
    async fn get_active_items(&self, user_id: Uuid) -> AppResult<Vec<Item>>;
    async fn get_archived_items(&self, user_id: Uuid) -> AppResult<Vec<Item>>;
    async fn get_deleted_items(&self, user_id: Uuid) -> AppResult<Vec<Item>>;

    // POST logic

    async fn create_item(&self, user_id: Uuid, id: Uuid, title: String, description: Option<String>, motivation: Option<i32>, due: DateTime<Utc>, duration_minutes: Option<i32>) -> AppResult<()>;
    // UPDATE logic
    async fn update_item_status(&self, user_id: Uuid, id: Uuid, status: TaskStatus) -> AppResult<()>;
    async fn update_sync_status(&self, user_id: Uuid, id: Uuid, sync_status: &str) -> AppResult<()>;

    async fn update_item_details(&self, user_id: Uuid, id: Uuid, title: String, description: Option<String>, due: DateTime<Utc>, duration_minutes: Option<i32>, motivation: Option<i32>) -> AppResult<()>;
    // ARCHIVE logic
    async fn archive_item(&self, user_id: Uuid, id: Uuid) -> AppResult<()>;
    async fn unarchive_item(&self, user_id: Uuid, id: Uuid) -> AppResult<()>;

    // SOFT DELETE logic
    async fn soft_delete_item(&self, user_id: Uuid, id: Uuid) -> AppResult<()>;
    async fn restore_item(&self, user_id: Uuid, id: Uuid) -> AppResult<()>;

    // HARD DELETE logic
    async fn hard_delete_item(&self, user_id: Uuid, id: Uuid) -> AppResult<()>;
    async fn empty_item_trash(&self, user_id: Uuid, full_wipe: bool) -> AppResult<()>;

    // CLAIM OFFLINE ITEMS - legacy safety for rows missing user_id
    async fn claim_offline_items(&self, user_id: Uuid) -> AppResult<usize>;

}