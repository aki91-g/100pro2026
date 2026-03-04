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
use crate::commands::auth_commands::*;
use crate::commands::debug::*;
use crate::database::connection::init_sqlite;
use crate::repositories::user_repo::UserRepository;
use crate::repositories::profile_repo::ProfileRepository;

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
    dotenvy::from_path(std::path::Path::new("../../../.env")).ok();
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
            app.manage(app_state.clone());
            
            let handle = app.handle().clone();
            
            // 3. Local SQLite
            let sqlite_pool = tauri::async_runtime::block_on(async {
                init_sqlite(&handle).await.expect("SQLite failed to initialize")
            });

            let sqlite_item_repo = Arc::new(crate::repositories::sqlite_item_repo::SqliteItemRepo { 
                pool: sqlite_pool.clone() 
            });

            // 3a. Initialize User Repository
            let user_repo: Arc<dyn UserRepository> = Arc::new(
                crate::repositories::sqlite_user_repo::SqliteUserRepo { 
                    pool: sqlite_pool.clone() 
                }
            );

            // 3b. Check for existing local user and auto-login
            let app_state_login = app_state.clone();
            let user_repo_login = user_repo.clone();
            tauri::async_runtime::block_on(async move {
                match user_repo_login.get_active_user().await {
                    Ok(Some(user)) => {
                        println!("🔐 Auto-login: Found active user {}", user.username);
                        app_state_login.set_user(user.id.clone(), user.username.clone()).await;
                        let _ = user_repo_login.update_last_login(&user.id).await;
                    }
                    Ok(None) => {
                        println!("👤 No active user found - offline mode available");
                    }
                    Err(e) => {
                        eprintln!("⚠️ Failed to check for active user: {}", e);
                    }
                }
            });

            // Register User Repository
            app.manage(user_repo.clone());

            // 4. Initialize Profile Repository to None (will be set when Postgres connects)
            app.manage(Arc::new(tokio::sync::RwLock::new(
                None::<Arc<dyn ProfileRepository>>
            )));

            // 5. Initialize Services (Start with Local only)
            let item_service = Arc::new(ItemService::new(sqlite_item_repo.clone(), None, handle.clone()));
            let debug_service = Arc::new(DebugService::new(sqlite_item_repo.clone(), None));

            // Register Services
            app.manage(item_service.clone());
            app.manage(debug_service.clone());

            // 6. Async Postgres Connection
            let item_service_bg = item_service.clone();
            let debug_service_bg = debug_service.clone();
            let app_handle_bg = handle.clone();

            tauri::async_runtime::spawn(async move {
                if let Some(pg_pool) = crate::database::connection::init_postgres().await {
                    let pg_item_repo = Arc::new(crate::repositories::postgres_item_repo::PostgresItemRepo { 
                        pool: pg_pool.clone()
                    });
                    
                    let pg_profile_repo: Arc<dyn ProfileRepository> = Arc::new(
                        crate::repositories::postgres_profile_repo::PostgresProfileRepo {
                            pool: pg_pool.clone()
                        }
                    );
                    
                    item_service_bg.set_remote(pg_item_repo.clone()).await;
                    debug_service_bg.set_remote(pg_item_repo).await;
                    
                    // Update the managed ProfileRepository
                    if let Some(profile_repo_lock) = app_handle_bg.try_state::<Arc<tokio::sync::RwLock<Option<Arc<dyn ProfileRepository>>>>>() {
                        let mut guard = profile_repo_lock.write().await;
                        *guard = Some(pg_profile_repo);
                    }
                    
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
            // auth_commands
            login,
            logout,
            get_active_local_user,
            auto_login,
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