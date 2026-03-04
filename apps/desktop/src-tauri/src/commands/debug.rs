use crate::services::debug_service::DebugService;
use tauri::State;
use std::sync::Arc;

type CommandResult<T> = Result<T, String>;

#[tauri::command]
pub async fn debug_reset_db(service: State<'_, Arc<DebugService>>) -> CommandResult<()> {
    service.inner().reset_all_databases().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn debug_seed_data(service: State<'_, Arc<DebugService>>) -> CommandResult<()> {
    service.inner().seed_test_data().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn debug_full_wipe_items(service: State<'_, Arc<DebugService>>) -> CommandResult<()> {
    service.inner().reset_all_databases().await.map_err(|e| e.to_string())
}