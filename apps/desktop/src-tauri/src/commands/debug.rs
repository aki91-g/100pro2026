use crate::error::{AppError, AppResult};
use crate::models::item::TaskStatus;
use uuid::Uuid;
use sqlx::SqlitePool;
use tauri::State;

#[tauri::command]
pub async fn debug_reset_db(pool: State<'_, SqlitePool>) -> AppResult<()> {
    if !cfg!(debug_assertions) {
        return Err(AppError::InvalidInput(
            "debug_reset_db is disabled in release builds".to_string(),
        ));
    }

    // 1. Wipe everything
    sqlx::query("DELETE FROM items").execute(&*pool).await?;

    // 2. Insert one of each state for testing
    let statuses = vec![TaskStatus::Todo, TaskStatus::InProgress, TaskStatus::Done];
    
    for (i, status) in statuses.into_iter().enumerate() {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO items (id, title, status, motivation, is_archived, created_at) 
             VALUES (?, ?, ?, ?, ?, datetime('now', ?))"
        )
        .bind(id)
        .bind(format!("Sample Task {}", i))
        .bind(status)
        .bind(5)
        .bind(0) // not archived
        .bind(format!("-{} minutes", i * 10))
        .execute(&*pool)
        .await?;
    }

    // 3. Insert one Archived
    sqlx::query("INSERT INTO items (id, title, status, is_archived) VALUES (?, 'Archived Task', 'done', 1)")
        .bind(Uuid::new_v4())
        .execute(&*pool).await?;

    // 4. Insert one Soft-Deleted
    sqlx::query("INSERT INTO items (id, title, status, deleted_at) VALUES (?, 'Deleted Task', 'todo', CURRENT_TIMESTAMP)")
        .bind(Uuid::new_v4())
        .execute(&*pool).await?;

    Ok(())
}