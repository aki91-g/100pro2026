use crate::services::debug_service::DebugService;
use crate::state::AppState;
use crate::database::connection::init_postgres;
use crate::repositories::postgres_item_repo::PostgresItemRepo;
use tauri::State;
use std::sync::Arc;

async fn ensure_remote_repo(service: &Arc<DebugService>) -> Result<(), String> {
    if service.has_remote().await {
        return Ok(());
    }

    let pg_pool = init_postgres()
        .await
        .ok_or_else(|| {
            "PostgreSQL is not connected. Clear data aborted to avoid local/remote mismatch. Check DATABASE_URL and network connectivity.".to_string()
        })?;

    let pg_item_repo = Arc::new(PostgresItemRepo { pool: pg_pool });
    service.set_remote(pg_item_repo).await;
    Ok(())
}

#[tauri::command]
pub async fn debug_reset_db(
    service: State<'_, Arc<DebugService>>,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    ensure_remote_repo(service.inner()).await?;

    let user_id = app_state.get_user_id().await
        .map_err(|_| "No user logged in. Please login first.".to_string())?;
    
    service.inner().reset_all_databases(&user_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn debug_seed_data(
    service: State<'_, Arc<DebugService>>,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let user_id = app_state.get_user_id().await
        .map_err(|_| "No user logged in. Please login first to seed data.".to_string())?;
    
    service.inner().seed_test_data(&user_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn debug_full_wipe_items(
    service: State<'_, Arc<DebugService>>,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    ensure_remote_repo(service.inner()).await?;

    let user_id = app_state.get_user_id().await
        .map_err(|_| "No user logged in. Please login first.".to_string())?;
    
    service.inner().reset_all_databases(&user_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn debug_migrate_null_users(
    service: State<'_, Arc<DebugService>>,
    app_state: State<'_, AppState>,
    assign_to_current_user: bool
) -> Result<usize, String> {
    let user_id = if assign_to_current_user {
        Some(app_state.get_user_id().await
            .map_err(|_| "No user logged in. Cannot assign to current user.".to_string())?)
    } else {
        None
    };
    
    service.inner().migrate_null_user_items(user_id.as_deref()).await.map_err(|e| e.to_string())
}