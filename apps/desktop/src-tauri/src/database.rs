use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::str::FromStr;
use tauri::{AppHandle, Manager}; 
use crate::error::{AppResult, AppError};

pub async fn setup_database(handle: &AppHandle) -> AppResult<SqlitePool> {
    let app_dir = handle.path().app_data_dir().map_err(|_| {
        AppError::PathNotFound("Failed to get app data directory".to_string())
    })?;

    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir)?;
    }

    let db_path = app_dir.join("app.db");
    let db_url = format!("sqlite:{}", db_path.to_string_lossy());

    let opt = SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

    let pool = SqlitePool::connect_with(opt).await?;

    // SQLファイルを実行（パスは src-tauri からの相対パス）
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    println!("Database path: {:?}", db_path);

    Ok(pool)
}