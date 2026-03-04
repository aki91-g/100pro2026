use crate::models::item::{Item, TaskStatus};
use crate::services::item_service::ItemService;
use crate::state::AppState;
use uuid::Uuid;
use tauri::State;
use chrono::{DateTime, Utc};
use std::sync::Arc;

// Helper to convert our internal AppResult to a String for the Frontend
type CommandResult<T> = Result<T, String>;

#[tauri::command]
pub async fn get_active_items(service: State<'_, Arc<ItemService>>, app_state: State<'_, AppState>) -> CommandResult<Vec<Item>> {
    let user_id = app_state.get_user_id().await?;
    service.get_active_items(&user_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_archived_items(service: State<'_, Arc<ItemService>>, app_state: State<'_, AppState>) -> CommandResult<Vec<Item>> {
    let user_id = app_state.get_user_id().await?;
    service.get_archived_items(&user_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_deleted_items(service: State<'_, Arc<ItemService>>, app_state: State<'_, AppState>) -> CommandResult<Vec<Item>> {
    let user_id = app_state.get_user_id().await?;
    service.get_deleted_items(&user_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_item(
    service: State<'_, Arc<ItemService>>,
    app_state: State<'_, AppState>,
    title: String, 
    motivation: i8, 
    due: Option<DateTime<Utc>>, 
    duration_minutes: Option<i32>, 
) -> CommandResult<Uuid> {
    let user_id = app_state.get_user_id().await?;
    service.create_item(&user_id, title, motivation, due, duration_minutes).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_item_status(service: State<'_, Arc<ItemService>>, app_state: State<'_, AppState>, id: Uuid, status: TaskStatus) -> CommandResult<()> {
    let user_id = app_state.get_user_id().await?;
    service.update_item_status(&user_id, id, status).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_item_details(
    service: State<'_, Arc<ItemService>>,
    app_state: State<'_, AppState>,
    id: Uuid,
    title: String,
    description: Option<String>,
    due: Option<DateTime<Utc>>,
    duration_minutes: Option<i32>,
    motivation: i8,
) -> CommandResult<()> {
    let user_id = app_state.get_user_id().await?;
    service.update_item_details(&user_id, id, title, description, due, duration_minutes, motivation).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn archive_item(service: State<'_, Arc<ItemService>>, app_state: State<'_, AppState>, id: Uuid) -> CommandResult<()> {
    let user_id = app_state.get_user_id().await?;
    service.archive_item(&user_id, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn unarchive_item(service: State<'_, Arc<ItemService>>, app_state: State<'_, AppState>, id: Uuid) -> CommandResult<()> {
    let user_id = app_state.get_user_id().await?;
    service.unarchive_item(&user_id, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn soft_delete_item(service: State<'_, Arc<ItemService>>, app_state: State<'_, AppState>, id: Uuid) -> CommandResult<()> {
    let user_id = app_state.get_user_id().await?;
    service.soft_delete_item(&user_id, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn restore_item(service: State<'_, Arc<ItemService>>, app_state: State<'_, AppState>, id: Uuid) -> CommandResult<()> {
    let user_id = app_state.get_user_id().await?;
    service.restore_item(&user_id, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn hard_delete_item(service: State<'_, Arc<ItemService>>, app_state: State<'_, AppState>, id: Uuid) -> CommandResult<()> {
    let user_id = app_state.get_user_id().await?;
    service.hard_delete_item(&user_id, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn empty_item_trash(service: State<'_, Arc<ItemService>>, app_state: State<'_, AppState>) -> CommandResult<()> {
    let user_id = app_state.get_user_id().await?;
    service.empty_item_trash(&user_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn claim_offline_items(service: State<'_, Arc<ItemService>>, app_state: State<'_, AppState>) -> CommandResult<usize> {
    let user_id = app_state.get_user_id().await?;
    service.claim_offline_items(&user_id).await.map_err(|e| e.to_string())
}