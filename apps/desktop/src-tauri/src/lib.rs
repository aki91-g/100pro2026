// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod error;
pub mod database;

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
            // handleをcloneしてasyncブロックに渡す
            let handle = app.handle().clone();

            // DBのセットアップ（async実行）
            // let pool = tauri::async_runtime::block_on(async move {
            //     database::setup_database(&handle)
            //         .await
            //         .expect("Failed to setup database")
            // });

            let pool = tauri::async_runtime::block_on(async move {
                // expect ではなく match でエラーを捕まえる
                match database::setup_database(&handle).await {
                    Ok(p) => p,
                    Err(e) => {
                        // ターミナルに赤字でエラーを出す
                        eprintln!("\n--- DATABASE SETUP ERROR ---\n{}\n----------------------------\n", e);
                        panic!("Check the error message above!"); // ここで落とす
                    }
                }
            });
            app.manage(pool);
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
