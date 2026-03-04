use crate::services::debug_service::DebugService;
use crate::state::AppState;
use tauri::State;
use std::sync::Arc;

type CommandResult<T> = Result<T, String>;

#[tauri::command]
pub async fn debug_reset_db(
    service: State<'_, Arc<DebugService>>,
    app_state: State<'_, AppState>,
) -> CommandResult<()> {
    let user_id = app_state.get_user_id().await
        .map_err(|_| "No user logged in. Please login first.".to_string())?;
    
    service.inner().reset_all_databases(&user_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn debug_seed_data(
    service: State<'_, Arc<DebugService>>,
    app_state: State<'_, AppState>,
) -> CommandResult<()> {
    let user_id = app_state.get_user_id().await
        .map_err(|_| "No user logged in. Please login first to seed data.".to_string())?;
    
    service.inner().seed_test_data(&user_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn debug_full_wipe_items(
    service: State<'_, Arc<DebugService>>,
    app_state: State<'_, AppState>,
) -> CommandResult<()> {
    let user_id = app_state.get_user_id().await
        .map_err(|_| "No user logged in. Please login first.".to_string())?;
    
    service.inner().reset_all_databases(&user_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn debug_migrate_null_users(
    service: State<'_, Arc<DebugService>>,
    app_state: State<'_, AppState>,
    assign_to_current_user: bool
) -> CommandResult<usize> {
    let user_id = if assign_to_current_user {
        Some(app_state.get_user_id().await
            .map_err(|_| "No user logged in. Cannot assign to current user.".to_string())?)
    } else {
        None
    };
    
    service.inner().migrate_null_user_items(user_id.as_deref()).await.map_err(|e| e.to_string())
}