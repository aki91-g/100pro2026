// cd apps/desktop/src-tauri
// cargo test --test db_test

use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use uuid::Uuid;

use desktop_lib::models::item::{Item, TaskStatus};
use desktop_lib::error::{AppResult, AppError};

async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();

    // Run migrations so the 'items' table exists in memory
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations for test");

    pool
}

#[tokio::test]
async fn test_trigger_updates_timestamp() -> AppResult<()> {
    let pool = setup_test_db().await;
    let id = Uuid::new_v4();

    // 1. Insert
    sqlx::query("INSERT INTO items (id, title, status) VALUES (?, 'Timestamp Test', ?)")
        .bind(id).bind(TaskStatus::Todo).execute(&pool).await?;

    let initial: Item = sqlx::query_as("SELECT * FROM items WHERE id = ?").bind(id).fetch_one(&pool).await?;
    
    // Sleep for 1 second to ensure the timestamp will actually be different
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // 2. Update
    sqlx::query("UPDATE items SET title = 'Updated' WHERE id = ?").bind(id).execute(&pool).await?;

    let updated: Item = sqlx::query_as("SELECT * FROM items WHERE id = ?").bind(id).fetch_one(&pool).await?;

    // 3. Assert
    assert!(updated.updated_at > initial.updated_at, "Trigger should have updated the timestamp");
    Ok(())
}

#[tokio::test]
async fn test_full_task_lifecycle() -> AppResult<()> {
    let pool = setup_test_db().await;
    let id = Uuid::new_v4();

    // --- 1. TEST INSERTION ---
    sqlx::query("INSERT INTO items (id, title, status, motivation) VALUES (?, ?, ?, ?)")
        .bind(id)
        .bind("Mistake Task")
        .bind(TaskStatus::Todo)
        .bind(5)
        .execute(&pool)
        .await?;

    // --- 2. TEST ARCHIVE LOGIC (Accidental Archive -> Restore) ---
    // Archive it
    sqlx::query("UPDATE items SET is_archived = 1 WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;

    let archived: Item = sqlx::query_as("SELECT * FROM items WHERE id = ?").bind(id).fetch_one(&pool).await?;
    assert!(archived.is_archived, "Task should be archived");

    // Unarchive it
    sqlx::query("UPDATE items SET is_archived = 0 WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;

    let unarchived: Item = sqlx::query_as("SELECT * FROM items WHERE id = ?").bind(id).fetch_one(&pool).await?;
    assert!(!unarchived.is_archived, "Task should be unarchived");

    // --- 3. TEST DATA MODIFICATION (Accidental Motivation/Title -> Edit) ---
    sqlx::query("UPDATE items SET title = ?, motivation = ? WHERE id = ?")
        .bind("Corrected Title")
        .bind(10)
        .bind(id)
        .execute(&pool)
        .await?;

    let edited: Item = sqlx::query_as("SELECT * FROM items WHERE id = ?").bind(id).fetch_one(&pool).await?;
    assert_eq!(edited.title, "Corrected Title");
    assert_eq!(edited.motivation, 10);

    // --- 4. TEST SOFT DELETE (Accidental Delete -> Restore) ---
    sqlx::query("UPDATE items SET deleted_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;

    let deleted: Item = sqlx::query_as("SELECT * FROM items WHERE id = ?").bind(id).fetch_one(&pool).await?;
    assert!(deleted.deleted_at.is_some());

    // Restore
    sqlx::query("UPDATE items SET deleted_at = NULL WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;

    let restored: Item = sqlx::query_as("SELECT * FROM items WHERE id = ?").bind(id).fetch_one(&pool).await?;
    assert!(restored.deleted_at.is_none());

    Ok(())
}

#[tokio::test]
async fn test_filtering_logic() -> AppResult<()> {
    let pool = setup_test_db().await;
    
    // Use the Enum to ensure type safety
    let status = TaskStatus::Todo;

    // 1. Insert 1 Active, 1 Archived, 1 Deleted (All need a 'status')
    sqlx::query("INSERT INTO items (id, title, status, is_archived, deleted_at) VALUES (?, 'Active', ?, 0, NULL)")
        .bind(Uuid::new_v4()).bind(&status).execute(&pool).await?;
        
    sqlx::query("INSERT INTO items (id, title, status, is_archived, deleted_at) VALUES (?, 'Archived', ?, 1, NULL)")
        .bind(Uuid::new_v4()).bind(&status).execute(&pool).await?;
        
    sqlx::query("INSERT INTO items (id, title, status, is_archived, deleted_at) VALUES (?, 'Deleted', ?, 0, CURRENT_TIMESTAMP)")
        .bind(Uuid::new_v4()).bind(&status).execute(&pool).await?;

    // 2. Test Active Filter
    let active: Vec<Item> = sqlx::query_as("SELECT * FROM items WHERE deleted_at IS NULL AND is_archived = 0")
        .fetch_all(&pool).await?;
    
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].title, "Active");

    Ok(())
}

#[tokio::test]
async fn test_not_found_error() -> AppResult<()> {
    let pool = setup_test_db().await;
    let fake_id = Uuid::new_v4();

    let result = sqlx::query("UPDATE items SET status = ? WHERE id = ?")
        .bind(TaskStatus::Done)
        .bind(fake_id)
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        // No 'crate::' prefix. Just use the imported type.
        let err = AppError::NotFound(fake_id.to_string());
        
        assert_eq!(err.to_string(), format!("Item not found: {fake_id}"));
        return Ok(());
    }

    panic!("Should have returned a NotFound error for a fake ID");
}

#[tokio::test]
async fn test_constraint_violation() -> AppResult<()> {
    let pool = setup_test_db().await;

    // 1. Attempt an invalid insertion (missing 'title' which is NOT NULL)
    // We expect this to return an Err from SQLx
    let result = sqlx::query("INSERT INTO items (id, status) VALUES (?, ?)")
        .bind(Uuid::new_v4())
        .bind(TaskStatus::Todo)
        .execute(&pool)
        .await;

    // 2. Assert it is an error
    assert!(result.is_err(), "Database should have rejected insertion without a title");
    
    // 3. Verify it maps correctly to our AppError::Database variant
    if let Err(e) = result {
        // Use the imported AppError instead of crate::error::AppError
        let app_err = AppError::from(e);
        
        // matches! is the most idiomatic way to check enum variants in tests
        assert!(
            matches!(app_err, AppError::Database(_)),
            "Expected AppError::Database, but got: {:?}", app_err
        );
    }

    Ok(())
}