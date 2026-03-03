use std::sync::Arc;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{error::AppResult, models::item::{Item, TaskStatus}, repositories::item_repo::ItemRepository};

pub struct ItemService {
    repo: Arc<dyn ItemRepository>,
}

impl ItemService {
    pub fn new(repo: Arc<dyn ItemRepository>) -> Self { Self { repo } }

    pub async fn create_item(&self, title: String, motivation: i8, due: Option<DateTime<Utc>>, duration_minutes: Option<i32>) -> AppResult<Uuid> {
        let id = Uuid::new_v4();
        // Step 1: Persist via configured repository backend
        self.repo.create_item(id, title, motivation, due, duration_minutes).await?;
        
        // Step 2: (Future) Emit domain/integration event
        
        Ok(id)
    }

    // Mirroring all other functions
    pub async fn get_active_items(&self) -> AppResult<Vec<Item>> { self.repo.get_active_items().await }
    pub async fn get_archived_items(&self) -> AppResult<Vec<Item>> { self.repo.get_archived_items().await }
    pub async fn get_deleted_items(&self) -> AppResult<Vec<Item>> { self.repo.get_deleted_items().await }
    pub async fn update_item_status(&self, id: Uuid, status: TaskStatus) -> AppResult<()> { self.repo.update_item_status(id, status).await }
    pub async fn update_item_details(&self, id: Uuid, title: String, description: Option<String>, due: Option<DateTime<Utc>>, duration_minutes: Option<i32>, motivation: i8) -> AppResult<()> {
        self.repo.update_item_details(id, title, description, due, duration_minutes, motivation).await
    }
    pub async fn archive_item(&self, id: Uuid) -> AppResult<()> { self.repo.archive_item(id).await }
    pub async fn unarchive_item(&self, id: Uuid) -> AppResult<()> { self.repo.unarchive_item(id).await }
    pub async fn soft_delete_item(&self, id: Uuid) -> AppResult<()> { self.repo.soft_delete_item(id).await }
    pub async fn restore_item(&self, id: Uuid) -> AppResult<()> { self.repo.restore_item(id).await }
    pub async fn hard_delete_item(&self, id: Uuid) -> AppResult<()> { self.repo.hard_delete_item(id).await }
    pub async fn empty_item_trash(&self) -> AppResult<()> { self.repo.empty_item_trash().await }
}