use crate::models::item::{Item, TaskStatus};
use crate::services::item_service::ItemService;
use uuid::Uuid;
use tauri::State;
use chrono::{DateTime, Utc};
use std::sync::Arc;

// Helper to convert our internal AppResult to a String for the Frontend
type CommandResult<T> = Result<T, String>;

#[tauri::command]
pub async fn get_active_items(service: State<'_, Arc<ItemService>>) -> CommandResult<Vec<Item>> {
    service.get_active_items().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_archived_items(service: State<'_, Arc<ItemService>>) -> CommandResult<Vec<Item>> {
    service.get_archived_items().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_deleted_items(service: State<'_, Arc<ItemService>>) -> CommandResult<Vec<Item>> {
    service.get_deleted_items().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_item(
    service: State<'_, Arc<ItemService>>,
    title: String, 
    motivation: i8, 
    due: Option<DateTime<Utc>>, 
    duration_minutes: Option<i32>, 
) -> CommandResult<Uuid> {
    service.create_item(title, motivation, due, duration_minutes).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_item_status(service: State<'_, Arc<ItemService>>, id: Uuid, status: TaskStatus) -> CommandResult<()> {
    service.update_item_status(id, status).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_item_details(
    service: State<'_, Arc<ItemService>>,
    id: Uuid,
    title: String,
    description: Option<String>,
    due: Option<DateTime<Utc>>,
    duration_minutes: Option<i32>,
    motivation: i8,
) -> CommandResult<()> {
    service.update_item_details(id, title, description, due, duration_minutes, motivation).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn archive_item(service: State<'_, Arc<ItemService>>, id: Uuid) -> CommandResult<()> {
    service.archive_item(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn unarchive_item(service: State<'_, Arc<ItemService>>, id: Uuid) -> CommandResult<()> {
    service.unarchive_item(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn soft_delete_item(service: State<'_, Arc<ItemService>>, id: Uuid) -> CommandResult<()> {
    service.soft_delete_item(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn restore_item(service: State<'_, Arc<ItemService>>, id: Uuid) -> CommandResult<()> {
    service.restore_item(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn hard_delete_item(service: State<'_, Arc<ItemService>>, id: Uuid) -> CommandResult<()> {
    service.hard_delete_item(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn empty_item_trash(service: State<'_, Arc<ItemService>>) -> CommandResult<()> {
    service.empty_item_trash().await.map_err(|e| e.to_string())
}