// Receive data from frontend, call a service, and return a result
use crate::error::AppResult;
use crate::models::item::{Item, TaskStatus};
use crate::services::item_service::ItemService;
use uuid::Uuid;
use tauri::State;
use chrono::{DateTime, Utc};

#[tauri::command]
pub async fn get_active_items(service: State<'_, ItemService>) -> AppResult<Vec<Item>> {
    service.get_active_items().await
}

#[tauri::command]
pub async fn get_archived_items(service: State<'_, ItemService>) -> AppResult<Vec<Item>> {
    service.get_archived_items().await
}

#[tauri::command]
pub async fn get_deleted_items(service: State<'_, ItemService>) -> AppResult<Vec<Item>> {
    service.get_deleted_items().await
}

#[tauri::command]
pub async fn create_item(
    service: State<'_, ItemService>,
    title: String, 
    motivation: i8, 
    due: Option<DateTime<Utc>>, 
    duration_minutes: Option<i32>, 
) -> AppResult<Uuid> {
    service.create_item(title, motivation, due, duration_minutes).await
}

#[tauri::command]
pub async fn update_item_status(service: State<'_, ItemService>, id: Uuid, status: TaskStatus) -> AppResult<()> {
    service.update_item_status(id, status).await
}

#[tauri::command]
pub async fn update_item_details(
    service: State<'_, ItemService>,
    id: Uuid,
    title: String,
    description: Option<String>,
    due: Option<DateTime<Utc>>,
    duration_minutes: Option<i32>,
    motivation: i8,
) -> AppResult<()> {
    service.update_item_details(id, title, description, due, duration_minutes, motivation).await
}

#[tauri::command]
pub async fn archive_item(service: State<'_, ItemService>, id: Uuid) -> AppResult<()> {
    service.archive_item(id).await
}

#[tauri::command]
pub async fn unarchive_item(service: State<'_, ItemService>, id: Uuid) -> AppResult<()> {
    service.unarchive_item(id).await
}

#[tauri::command]
pub async fn soft_delete_item(service: State<'_, ItemService>, id: Uuid) -> AppResult<()> {
    service.soft_delete_item(id).await
}

#[tauri::command]
pub async fn restore_item(service: State<'_, ItemService>, id: Uuid) -> AppResult<()> {
    service.restore_item(id).await
}

#[tauri::command]
pub async fn hard_delete_item(service: State<'_, ItemService>, id: Uuid) -> AppResult<()> {
    service.hard_delete_item(id).await
}

#[tauri::command]
pub async fn empty_item_trash(service: State<'_, ItemService>) -> AppResult<()> {
    service.empty_item_trash().await
}