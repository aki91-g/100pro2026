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
    println!("🗄️ SQLite database path: {:?}", db_path);
    println!("🗄️ SQLite connection URL: {}", db_url);

    // 2. Connect with options (Enables foreign keys and WAL mode for speed)
    let options = SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

    let pool = SqlitePool::connect_with(options).await?;

    // 3. Run migrations (Verifies: pool → same file → migrations applied)
    println!("🔄 Running SQLite migrations from ./migrations/sqlite...");
    if let Err(e) = sqlx::migrate!("./migrations/sqlite").run(&pool).await {
        eprintln!("❌ MIGRATION ERROR: {:?}", e);
        return Err(e.into());
    }
    println!("📦 SQLite migrations finished successfully");

    // 4. Normalize legacy UUID BLOB values to TEXT (from earlier bind behavior)
    // This prevents decode errors like: String/TEXT incompatible with SQL type BLOB.
    let _ = sqlx::query(
        "UPDATE local_user
         SET id = lower(
            hex(substr(id,1,4)) || '-' ||
            hex(substr(id,5,2)) || '-' ||
            hex(substr(id,7,2)) || '-' ||
            hex(substr(id,9,2)) || '-' ||
            hex(substr(id,11,6))
         )
         WHERE typeof(id) = 'blob' AND length(id) = 16"
    )
    .execute(&pool)
    .await;

    let _ = sqlx::query(
        "UPDATE items
         SET id = lower(
            hex(substr(id,1,4)) || '-' ||
            hex(substr(id,5,2)) || '-' ||
            hex(substr(id,7,2)) || '-' ||
            hex(substr(id,9,2)) || '-' ||
            hex(substr(id,11,6))
         )
         WHERE typeof(id) = 'blob' AND length(id) = 16"
    )
    .execute(&pool)
    .await;

    let _ = sqlx::query(
        "UPDATE items
         SET user_id = lower(
            hex(substr(user_id,1,4)) || '-' ||
            hex(substr(user_id,5,2)) || '-' ||
            hex(substr(user_id,7,2)) || '-' ||
            hex(substr(user_id,9,2)) || '-' ||
            hex(substr(user_id,11,6))
         )
         WHERE typeof(user_id) = 'blob' AND length(user_id) = 16"
    )
    .execute(&pool)
    .await;
    
    // 5. Verify the database file exists
    if db_path.exists() {
        println!("✅ Confirmed: Database file exists at {:?}", db_path);
    } else {
        eprintln!("❌ ERROR: Database file NOT found at {:?}", db_path);
    }
    
    // Query schema to verify both tables were created
    let tables: Vec<(String,)> = match sqlx::query_as(
        "SELECT name FROM sqlite_master WHERE type='table' AND name IN ('items', 'local_user')"
    )
    .fetch_all(&pool)
    .await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("⚠️  Could not verify tables: {}", e);
            vec![]
        }
    };
    
    let table_names: Vec<&str> = tables.iter().map(|t| t.0.as_str()).collect();
    println!("📋 Tables verified: {:?}", table_names);
    
    if !table_names.contains(&"local_user") {
        eprintln!("❌ CRITICAL: 'local_user' table NOT created! Pool may point to different file.");
    }
    if !table_names.contains(&"items") {
        eprintln!("❌ CRITICAL: 'items' table NOT created! Pool may point to different file.");
    }
    
    Ok(pool)
}

pub async fn init_postgres() -> Option<sqlx::PgPool> {
    let url = match std::env::var("DIRECT_URL") {
        Ok(v) => v,
        Err(e) => {
            eprintln!("DIRECT_URL not set: {}", e);
            return None;
        }
    };

    // 1. Configure the pool with the "PgBouncer-safe" settings
    let options = match sqlx::postgres::PgConnectOptions::from_str(&url) {
        Ok(opts) => opts.statement_cache_capacity(0),
        Err(e) => {
            eprintln!("Invalid DIRECT_URL: {}", e);
            return None;
        }
    }
        .statement_cache_capacity(0); 

    let pool = match sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_with(options) 
        .await
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Postgres connection failed: {}", e);
            return None;
        }
    };

    // 2. Run migrations using the POOL. 
    if let Err(e) = sqlx::migrate!("./migrations/postgres").run(&pool).await {
        eprintln!("Postgres migration error: {}", e);
        return None;
    }

    println!("REMOTE ACTIVE: Ready for seeding!");
    Some(pool)
}