// Receive data from frontend, call a service, and return a result
use crate::error::AppResult;
use crate::models::item::Item;
use crate::models::item::TaskStatus;
use uuid::Uuid;
use sqlx::SqlitePool;
use tauri::State;
use chrono::{DateTime, Utc};

// --- GET LOGIC (active, archived, soft deleted) ---
#[tauri::command]
pub async fn get_active_items(pool: tauri::State<'_, sqlx::SqlitePool>) -> AppResult<Vec<Item>> {
    // filter for tasks that are NOT deleted and NOT archived
    let items = sqlx::query_as::<_, Item>(
        "SELECT * FROM items 
         WHERE deleted_at IS NULL AND is_archived = 0 
         ORDER BY (due IS NULL), due ASC, created_at DESC"
    )
    .fetch_all(&*pool)
    .await?;
    Ok(items)
}

#[tauri::command]
pub async fn get_archived_items(pool: tauri::State<'_, sqlx::SqlitePool>) -> AppResult<Vec<Item>> {
    // filter for tasks that are NOT deleted and ARE archived
    let items = sqlx::query_as::<_, Item>(
        "SELECT * FROM items 
         WHERE deleted_at IS NULL AND is_archived = 1 
         ORDER BY created_at DESC"
    )
    .fetch_all(&*pool)
    .await?;
    Ok(items)
}

#[tauri::command]
pub async fn get_deleted_items(pool: tauri::State<'_, sqlx::SqlitePool>) -> AppResult<Vec<Item>> {
    // Show only items where deleted_at is SET
    let items = sqlx::query_as::<_, Item>(
        "SELECT * FROM items 
         WHERE deleted_at IS NOT NULL 
         ORDER BY deleted_at DESC"
    )
    .fetch_all(&*pool)
    .await?;
    Ok(items)
}

// --- POST LOGIC ---

#[tauri::command]
pub async fn create_item(
    title: String, 
    motivation: i8, 
    due: Option<DateTime<Utc>>, 
    duration_minutes: Option<i32>, 
    pool: State<'_, SqlitePool>
) -> AppResult<Uuid> {
    let id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO items (
            id, 
            title, 
            due, 
            duration_minutes, 
            status, 
            motivation, 
            is_archived
        ) 
        VALUES (?, ?, ?, ?, 'todo', ?, 0)"
    )
    .bind(id)               // 1st ?
    .bind(title)            // 2nd ?
    .bind(due)              // 3rd ? (SQLX handles Option as NULL if None)
    .bind(duration_minutes) // 4th ?
    .bind(motivation)       // 5th ?
    .execute(&*pool)
    .await?;
    
    Ok(id)
}



// --- UPDATE LOGIC (item status,all details) ---
#[tauri::command]
pub async fn update_item_status(
    id: Uuid, 
    status: TaskStatus, // Use the Enum here
    pool: State<'_, SqlitePool>
) -> AppResult<()> {
    sqlx::query("UPDATE items SET status = ? WHERE id = ?") // updated_at is handled by trigger now!
        .bind(status)
        .bind(id)
        .execute(&*pool)
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn update_item_details(
    id: Uuid,
    title: String,
    description: Option<String>,
    due: Option<DateTime<Utc>>,
    duration_minutes: Option<i32>,
    motivation: i8,
    pool: State<'_, SqlitePool>,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE items 
         SET title = ?, 
             description = ?, 
             due = ?, 
             duration_minutes = ?, 
             motivation = ? 
         WHERE id = ?"
    )
    .bind(title)            // 1
    .bind(description)      // 2
    .bind(due)              // 3
    .bind(duration_minutes) // 4
    .bind(motivation)       // 5
    .bind(id)               // 6
    .execute(&*pool)
    .await?;

    Ok(())
}


// --- ARCHIVE LOGIC ---
#[tauri::command]
pub async fn archive_item(id: Uuid, pool: State<'_, SqlitePool>) -> AppResult<()> {
    // We set is_archived to true. We don't touch deleted_at.
    sqlx::query(
        "UPDATE items 
        SET is_archived = 1, updated_at = CURRENT_TIMESTAMP 
        WHERE id = ? AND deleted_at IS NULL"
    )
    .bind(id)
    .execute(&*pool)
    .await?;
    Ok(())
}

#[tauri::command]
pub async fn unarchive_item(id: Uuid, pool: State<'_, SqlitePool>) -> AppResult<()> {
    sqlx::query(
        "UPDATE items 
        SET is_archived = 0, updated_at = CURRENT_TIMESTAMP 
        WHERE id = ? AND deleted_at IS NULL"
    )
    .bind(id)
    .execute(&*pool)
    .await?;
    Ok(())
}

// --- SOFT DELETE & RESTORE LOGIC ---
#[tauri::command]
pub async fn soft_delete_item(id: Uuid, pool: tauri::State<'_, sqlx::SqlitePool>) -> AppResult<()> {
    sqlx::query(
        "UPDATE items 
        SET deleted_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP 
        WHERE id = ?"
    )
    .bind(id)
    .execute(&*pool)
    .await?;
    Ok(())
}

#[tauri::command]
pub async fn restore_item(id: Uuid, pool: State<'_, SqlitePool>) -> AppResult<()> {
    // "Restore" by clearing the timestamp. It returns to its previous state (Active or Archived).
    sqlx::query("UPDATE items SET deleted_at = NULL, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(id)
        .execute(&*pool)
        .await?;
    Ok(())
}

// --- PERMANENT DELETE ---
#[tauri::command]
pub async fn hard_delete_item(id: Uuid, pool: State<'_, SqlitePool>) -> AppResult<()> {
    // Only use this for "Empty Trash" functionality.
    sqlx::query("DELETE FROM items WHERE id = ?")
        .bind(id)
        .execute(&*pool)
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn empty_trash(pool: State<'_, SqlitePool>) -> AppResult<()> {
    sqlx::query("DELETE FROM items WHERE deleted_at IS NOT NULL")
        .execute(&*pool)
        .await?;
    Ok(())
}