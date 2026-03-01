use sqlx::SqlitePool;
use uuid::Uuid;
// Ensure this matches the name in your Cargo.toml [lib] section
use desktop_lib::models::item::{Item, TaskStatus};
use desktop_lib::error::AppResult;

async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    pool
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