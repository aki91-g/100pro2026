// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod error;

pub mod commands;
pub mod database;
pub mod models;
pub mod repositories;
pub mod services;
pub mod utils;

use tauri::Manager;

use crate::commands::db_commands::*; 
#[cfg(debug_assertions)]
use crate::commands::debug::*; 
use crate::database::connection::init_db;

#[tauri::command]
fn is_dev() -> bool {
    cfg!(debug_assertions)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // 1. Initialize Logger
            let guard = crate::utils::logger::init(app.handle())
                .map_err(|e| e.to_string())?;
            
            // 2. Manage the guard so it stays alive for the duration of the app
            app.manage(guard); 

            // 3. Initialize Database
            let handle = app.handle().clone();
            
            // use block_on here to ensure the DB is ready before the window opens
            let result: crate::error::AppResult<()> = tauri::async_runtime::block_on(async move {
                let pool = init_db(&handle).await?;
                let repo = std::sync::Arc::new(crate::repositories::sqlite_item_repo::SqliteItemRepo { 
                    pool: pool.clone() 
                });
                let item_service = crate::services::item_service::ItemService::new(repo);
                handle.manage(pool);
                handle.manage(item_service);
                Ok(())
            });

            result.map_err(|e| e.to_string())?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // simple commands
            is_dev,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
