use async_trait::async_trait;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::item::{Item, TaskStatus};
use crate::error::AppResult;

#[async_trait]
pub trait ItemRepository: Send + Sync {
    // GET logic
    async fn get_active_items(&self) -> AppResult<Vec<Item>>;
    async fn get_archived_items(&self) -> AppResult<Vec<Item>>;
    async fn get_deleted_items(&self) -> AppResult<Vec<Item>>;

    // POST logic
    async fn create_item(&self, id: Uuid, title: String, motivation: i8, due: Option<DateTime<Utc>>, duration_minutes: Option<i32>) -> AppResult<()>;

    // UPDATE logic
    async fn update_item_status(&self, id: Uuid, status: TaskStatus) -> AppResult<()>;
    async fn update_item_details(&self, id: Uuid, title: String, description: Option<String>, due: Option<DateTime<Utc>>, duration_minutes: Option<i32>, motivation: i8) -> AppResult<()>;

    // ARCHIVE logic
    async fn archive_item(&self, id: Uuid) -> AppResult<()>;
    async fn unarchive_item(&self, id: Uuid) -> AppResult<()>;

    // SOFT DELETE logic
    async fn soft_delete_item(&self, id: Uuid) -> AppResult<()>;
    async fn restore_item(&self, id: Uuid) -> AppResult<()>;

    // HARD DELETE logic
    async fn hard_delete_item(&self, id: Uuid) -> AppResult<()>;
    async fn empty_item_trash(&self) -> AppResult<()>;
}