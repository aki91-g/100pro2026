// handle physical file creation and the connection pool
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::fs;
use std::str::FromStr;
use tauri::AppHandle;
use tauri::Manager;

pub async fn init_sqlite(app_handle: &AppHandle) -> crate::error::AppResult<SqlitePool> {
    // 1. Get the path to the app data directory
    let app_dir = app_handle.path().app_data_dir()?;
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)?;
    }

    let db_path = app_dir.join("tasks.db");
    let db_url = format!("sqlite:{}", db_path.display());

    // 2. Connect with options (Enables foreign keys and WAL mode for speed)
    let options = SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

    let pool = SqlitePool::connect_with(options).await?;

    // 3. Run migrations (Assumes you have a /migrations folder)
    sqlx::migrate!("./migrations/sqlite").run(&pool).await?;

    Ok(pool)
}

pub async fn init_postgres() -> Option<sqlx::PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;

    // 1. Configure the pool with the "PgBouncer-safe" settings
    let options = sqlx::postgres::PgConnectOptions::from_str(&url).ok()?
        .statement_cache_capacity(0); 

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_with(options) 
        .await
        .ok()?;

    // 2. Run migrations using the POOL. 
    if let Err(e) = sqlx::migrate!("./migrations/postgres").run(&pool).await {
        if !e.to_string().contains("prepared statement") {
            eprintln!("Postgres migration error: {}", e);
            return None;
        }
    }

    println!("REMOTE ACTIVE: Ready for seeding!");
    Some(pool)
}