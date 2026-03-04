// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod error;
pub mod commands;
pub mod database;
pub mod models;
pub mod repositories;
pub mod services;
pub mod state;
pub mod utils;

use std::sync::Arc;
use tauri::Manager;

// Cleaned up imports
use crate::state::AppState;
use crate::services::{debug_service::DebugService, item_service::ItemService};
use crate::commands::db_commands::*;
use crate::database::connection::init_sqlite;

#[cfg(debug_assertions)]
use crate::commands::debug::*;

#[tauri::command]
fn is_dev() -> bool {
    cfg!(debug_assertions)
}

// ensure they use State<'_, AppState>
#[tauri::command]
async fn sync_items(service: tauri::State<'_, Arc<ItemService>>, app_state: tauri::State<'_, AppState>) -> Result<usize, String> {
    let user_id = app_state.get_user_id().await?;
    service.sync_items(&user_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_user(app_state: tauri::State<'_, AppState>, user_id: String) -> Result<(), String> {
    app_state.set_user_id(user_id).await;
    Ok(())
}

#[tauri::command]
async fn get_current_user(app_state: tauri::State<'_, AppState>) -> Result<Option<String>, String> {
    match app_state.get_user_id().await {
        Ok(user_id) => Ok(Some(user_id)),
        Err(_) => Ok(None),
    }
}

#[tauri::command]
async fn clear_user(app_state: tauri::State<'_, AppState>) -> Result<(), String> {
    app_state.clear_user_id().await;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenvy::dotenv().ok();
    // Useful for Linux/Wayland dev environments
    std::env::set_var("WLR_NO_HARDWARE_CURSORS", "1");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // 1. Initialize Logger
            let guard = crate::utils::logger::init(app.handle()).map_err(|e| e.to_string())?;
            app.manage(guard); 

            // 2. Initialize AppState (CRITICAL: Use app.manage directly)
            let app_state = AppState::new();
            app.manage(app_state);
            
            let handle = app.handle().clone();
            
            // 3. Local SQLite
            let sqlite_pool = tauri::async_runtime::block_on(async {
                init_sqlite(&handle).await.expect("SQLite failed to initialize")
            });

            let sqlite_repo = Arc::new(crate::repositories::sqlite_item_repo::SqliteItemRepo { 
                pool: sqlite_pool.clone() 
            });

            // 4. Initialize Services (Start with Local only)
            let item_service = Arc::new(ItemService::new(sqlite_repo.clone(), None, handle.clone()));
            let debug_service = Arc::new(DebugService::new(sqlite_repo.clone(), None));

            // Register Services
            app.manage(item_service.clone());
            app.manage(debug_service.clone());

            // 5. Async Postgres Connection
            let item_service_bg = item_service.clone();
            let debug_service_bg = debug_service.clone();

            tauri::async_runtime::spawn(async move {
                if let Some(pg_pool) = crate::database::connection::init_postgres().await {
                    let pg_repo = Arc::new(crate::repositories::postgres_item_repo::PostgresItemRepo { 
                        pool: pg_pool 
                    });
                    
                    item_service_bg.set_remote(pg_repo.clone()).await;
                    debug_service_bg.set_remote(pg_repo).await;
                    
                    println!("🚀 Supabase connected & Remote Repositories activated.");
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            is_dev,
            sync_items,
            set_user,
            get_current_user,
            clear_user,
            // db_commands
            get_active_items,
            get_archived_items,
            get_deleted_items,
            create_item,
            update_item_status,
            update_item_details,
            archive_item,
            unarchive_item,
            soft_delete_item,
            restore_item,
            hard_delete_item,
            empty_item_trash,
            claim_offline_items,
            // debug commands
            debug_reset_db,
            debug_seed_data,
            debug_full_wipe_items,
            debug_migrate_null_users,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}