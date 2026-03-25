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
use std::path::PathBuf;
use tauri::Manager;
use uuid::Uuid;

// Cleaned up imports
use crate::state::AppState;
use crate::services::item_service::ItemService;
use crate::commands::db_commands::*;
use crate::commands::auth_commands::*;
use crate::database::connection::init_sqlite;
use crate::repositories::user_repo::UserRepository;
use crate::repositories::session_repo::SessionRepository;
use crate::repositories::profile_repo::ProfileRepository;

// ensure they use State<'_, AppState>
#[tauri::command]
async fn sync_items(service: tauri::State<'_, Arc<ItemService>>, app_state: tauri::State<'_, AppState>) -> Result<usize, String> {
    let user_id = app_state.get_user_id().await?;
    service.sync_items(user_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_user(app_state: tauri::State<'_, AppState>, user_id: Uuid) -> Result<(), String> {
    app_state.set_user_id(user_id).await;
    Ok(())
}

#[tauri::command]
async fn get_current_user(app_state: tauri::State<'_, AppState>) -> Result<Option<String>, String> {
    match app_state.get_user_id().await {
        Ok(user_id) => Ok(Some(user_id.to_string())),
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
    load_env_for_desktop();
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
                // This MUST finish migrations before returning the pool
                init_sqlite(&handle).await.expect("SQLite failed to initialize")
            });

            let sqlite_item_repo = Arc::new(crate::repositories::sqlite_item_repo::SqliteItemRepo { 
                pool: sqlite_pool.clone() 
            });

            // Expose sqlite pool for transactional auth operations.
            app.manage(sqlite_pool.clone());

            let user_repo: Arc<dyn UserRepository> = Arc::new(
                crate::repositories::sqlite_user_repo::SqliteUserRepo { 
                    pool: sqlite_pool.clone() 
                }
            );

            let session_repo: Arc<dyn SessionRepository> = Arc::new(
                crate::repositories::sqlite_session_repo::SqliteSessionRepo {
                    pool: sqlite_pool.clone()
                }
            );

            // 3b. Check for existing session and auto-login
            let app_state_login = app_state.clone();
            let session_repo_login = session_repo.clone();

            tauri::async_runtime::block_on(async move {
                match session_repo_login.get_active_session().await {
                    Ok(Some(session)) => {
                        println!("🔐 Auto-login: Found active session");
                        app_state_login.set_user(session.user_id, session.username.clone()).await;
                        if let Err(e) = session_repo_login.update_last_login().await {
                            eprintln!("⚠️ Auto-login update_last_login failed: {}", e);
                        }
                    }
                
                    Ok(None) => {
                        println!("👤 No active session found - login required");
                    }
                    Err(e) => {
                        eprintln!("⚠️ Failed to check for active session: {}", e);
                    }
                }
            });

            // Register User and Session Repositories
            app.manage(user_repo.clone());
            app.manage(session_repo.clone());

            // 4. Initialize Profile Repository to None (will be set when Postgres connects)
            app.manage(Arc::new(tokio::sync::RwLock::new(
                None::<Arc<dyn ProfileRepository>>
            )));

            // 5. Initialize Services (Start with Local only)
            let item_service = Arc::new(ItemService::new(sqlite_item_repo.clone(), None, handle.clone()));

            // Register Services
            app.manage(item_service.clone());

            // 6. Async Postgres Connection
            let item_service_bg = item_service.clone();
            let app_handle_bg = handle.clone();
            let app_state_bg = app_state.clone();

            tauri::async_runtime::spawn(async move {
                loop {
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
                        
                        // Update the managed ProfileRepository
                        if let Some(profile_repo_lock) = app_handle_bg.try_state::<Arc<tokio::sync::RwLock<Option<Arc<dyn ProfileRepository>>>>>() {
                            let mut guard = profile_repo_lock.write().await;
                            *guard = Some(pg_profile_repo);
                        }

                        if let Ok(user_id) = app_state_bg.get_user_id().await {
                            match item_service_bg.sync_local_to_remote(user_id).await {
                                Ok(count) => {
                                    println!("⬆️ Synced {} local items to remote for user {}", count, user_id);
                                }
                                Err(e) => {
                                    eprintln!("⚠️ Failed to sync local items to remote on connect: {}", e);
                                }
                            }
                        }
                        
                        println!("🚀 Supabase connected & Remote Repositories activated.");
                        break;
                    }

                    eprintln!("⏳ Postgres unavailable, retrying connection in 10s...");
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            sync_items,
            set_user,
            get_current_user,
            clear_user,
            // auth_commands
            login,
            register_local_user,
            logout,
            get_active_session,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn load_env_for_desktop() {
    use std::collections::HashSet;

    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join(".env"));
        candidates.push(cwd.join("..").join(".env"));
        candidates.push(cwd.join("..").join("..").join(".env"));
        candidates.push(cwd.join("..").join("..").join("..").join(".env"));
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    candidates.push(manifest_dir.join(".env"));
    candidates.push(manifest_dir.join("..").join(".env"));
    candidates.push(manifest_dir.join("..").join("..").join(".env"));
    candidates.push(manifest_dir.join("..").join("..").join("..").join(".env"));

    // Canonicalize and deduplicate to avoid checking duplicate paths
    let mut seen = HashSet::new();
    candidates = candidates
        .into_iter()
        .filter_map(|path| {
            let canonical = std::fs::canonicalize(&path).unwrap_or(path);
            if seen.insert(canonical.clone()) {
                Some(canonical)
            } else {
                None
            }
        })
        .collect();

    let mut loaded_any = false;
    for path in candidates {
        if !path.exists() {
            continue;
        }

        if dotenvy::from_path(&path).is_ok() {
            println!("Loaded environment from {}", path.display());
            loaded_any = true;
            break;
        }
    }

    if !loaded_any {
        eprintln!("No .env file loaded for desktop runtime; using process environment only.");
    }
}