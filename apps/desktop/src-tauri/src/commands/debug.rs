use crate::error::AppResult;
use crate::models::item::TaskStatus;
use uuid::Uuid;
use sqlx::SqlitePool;
use tauri::State;
use chrono::Utc;

#[tauri::command]
pub async fn debug_reset_db(pool: State<'_, SqlitePool>) -> AppResult<()> {
    #[cfg(not(debug_assertions))]
    return Err(AppError::InvalidInput("Disabled in release builds".into()));

    sqlx::query("DELETE FROM items").execute(&*pool).await?;
    Ok(())
}

#[tauri::command]
pub async fn debug_seed_data(pool: State<'_, SqlitePool>) -> AppResult<()> {
    #[cfg(not(debug_assertions))]
    return Err(AppError::InvalidInput("Disabled in release builds".into()));

    // 1. Clear existing data first so we don't duplicate on every click
    sqlx::query("DELETE FROM items").execute(&*pool).await?;

    let now = Utc::now();

    // Define a robust set of test cases
    // Format: (Title, Desc, Status, Motivation, is_archived, is_deleted)
    let seed_configs = vec![
        ("Backlog Item", "Planning stage", TaskStatus::Backlog, 0, false, false),
        ("InProgress Task", "Working on this", TaskStatus::InProgress, 5, false, false),
        ("Finished Task", "Ready to be archived", TaskStatus::Done, 2, false, false),
        ("Archived Project", "Past work", TaskStatus::Done, 0, true, false),
        ("Ghost Task", "This was deleted", TaskStatus::Todo, 0, false, true),
    ];

    for (title, desc, status, motivation, archived, deleted) in seed_configs {
        let deleted_at = if deleted { Some(now) } else { None };
        
        sqlx::query(
            "INSERT INTO items (
                id, title, description, status, motivation, 
                is_archived, created_at, updated_at, deleted_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(Uuid::new_v4())
        .bind(title)
        .bind(Some(desc))
        .bind(status)
        .bind(motivation)
        .bind(archived)
        .bind(now)
        .bind(now)
        .bind(deleted_at)
        .execute(&*pool)
        .await?;
    }

    Ok(())
}