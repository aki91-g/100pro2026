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
#[tauri::command]
pub async fn debug_seed_data(pool: State<'_, SqlitePool>) -> AppResult<()> {
    let now = chrono::Utc::now();
    let tasks = vec![
        (Uuid::new_v4(), "Buy Groceries", Some("Milk, eggs, and bread"), TaskStatus::Todo, 0),
        (Uuid::new_v4(), "Finish Tauri Project", Some("Implement the seed function"), TaskStatus::InProgress, 1),
        (Uuid::new_v4(), "Workout", Some("Go for a 5km run"), TaskStatus::Backlog, 0),
        (Uuid::new_v4(), "Old Task", Some("This is already done"), TaskStatus::Done, 0),
    ];

    for (id, title, desc, status, motivation) in tasks {
        sqlx::query(
            "INSERT INTO items (id, title, description, status, motivation, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(id)         // sqlx handles Uuid directly now
        .bind(title)
        .bind(desc)
        .bind(status)
        .bind(motivation)
        .bind(now)        // Bind chrono::DateTime<Utc>
        .bind(now)
        .execute(&*pool)
        .await?;
    }

    Ok(())
}