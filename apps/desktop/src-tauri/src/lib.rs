// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod error;

pub mod commands;
pub mod database;
pub mod models;
pub mod repositories;
pub mod services;
pub mod utils;

use std::sync::Arc;

use tauri::Manager;

use crate::{commands::db_commands::*, services::{debug_service::DebugService, item_service::ItemService}}; 
#[cfg(debug_assertions)]
use crate::commands::debug::*; 
use crate::database::connection::init_sqlite;

#[tauri::command]
fn is_dev() -> bool {
    cfg!(debug_assertions)
}

#[tauri::command]
async fn sync_items(service: tauri::State<'_, Arc<ItemService>>) -> Result<usize, String> {
    service.sync_items().await.map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenvy::dotenv().ok();
    std::env::set_var("WLR_NO_HARDWARE_CURSORS", "1");
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // 1. Logger
            let guard = crate::utils::logger::init(app.handle()).map_err(|e| e.to_string())?;
            app.manage(guard); 

            let handle = app.handle().clone();
            
            // 2. Local SQLite (Must block so we have a DB to start with)
            let sqlite_pool = tauri::async_runtime::block_on(async {
                init_sqlite(&handle).await.expect("SQLite failed")
            });
            let sqlite_repo = std::sync::Arc::new(crate::repositories::sqlite_item_repo::SqliteItemRepo { 
                pool: sqlite_pool.clone() 
            });

            // 3. Initialize both services with None initially
            let item_service = Arc::new(ItemService::new(sqlite_repo.clone(), None, handle.clone()));
            let debug_service = Arc::new(DebugService::new(sqlite_repo.clone(), None));

            handle.manage(item_service.clone());
            handle.manage(debug_service.clone()); // Don't forget to manage the DebugService Arc!

            // 4. Background Postgres Connection
            let item_service_bg = item_service.clone();
            let debug_service_bg = debug_service.clone();

            tauri::async_runtime::spawn(async move {
                if let Some(pg_pool) = crate::database::connection::init_postgres().await {
                    let pg_repo = std::sync::Arc::new(crate::repositories::postgres_item_repo::PostgresItemRepo { 
                        pool: pg_pool 
                    });
                    
                    // ACTIVATE remote for both!
                    item_service_bg.set_remote(pg_repo.clone()).await;
                    debug_service_bg.set_remote(pg_repo).await;
                    
                    println!("Supabase connected: Services updated.");
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // simple commands
            is_dev,
            sync_items,
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
            // debug
            #[cfg(debug_assertions)]
            debug_reset_db,
            #[cfg(debug_assertions)]
            debug_seed_data,
            #[cfg(debug_assertions)]
            debug_full_wipe_items,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
