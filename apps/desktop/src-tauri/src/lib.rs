// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod error;
pub mod database;
pub mod logger;

use tauri::Manager;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            // Initialize logger and manage the guard to keep it alive
            if let Ok(guard) = logger::init(app.handle()) {
                app.manage(guard);
            } else {
                eprintln!("Failed to initialize logger");
            }

            // Clone handle and pass to async block
            let handle = app.handle().clone();

            let pool = tauri::async_runtime::block_on(async move {
                // Use match to catch errors instead of expect
                match database::setup_database(&handle).await {
                    Ok(p) => p,
                    Err(e) => {
                        // Output error to stderr
                        eprintln!("\n--- DATABASE SETUP ERROR ---\n{}\n----------------------------\n", e);
                        panic!("Check the error message above!");
                    }
                }
            });
            app.manage(pool);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
