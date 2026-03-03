use async_trait::async_trait;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::item::{Item, TaskStatus};
use crate::error::AppResult;
use crate::repositories::item_repo::ItemRepository;

pub struct SqliteItemRepo { pub pool: sqlx::SqlitePool }

#[async_trait]
impl ItemRepository for SqliteItemRepo {
    async fn get_all_items(&self) -> AppResult<Vec<Item>> {
        let items = sqlx::query_as::<_, Item>(
            "SELECT * FROM items ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool).await?;
        Ok(items)
    }

    async fn get_active_items(&self) -> AppResult<Vec<Item>> {
        let items = sqlx::query_as::<_, Item>(
            "SELECT * FROM items 
             WHERE deleted_at IS NULL AND is_archived = 0 
             ORDER BY (due IS NULL), due ASC, created_at DESC"
        )
        .fetch_all(&self.pool).await?;
        Ok(items)
    }

    async fn get_archived_items(&self) -> AppResult<Vec<Item>> {
        let items = sqlx::query_as::<_, Item>(
            "SELECT * FROM items 
             WHERE deleted_at IS NULL AND is_archived = 1 
             ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool).await?;
        Ok(items)
    }

    async fn get_deleted_items(&self) -> AppResult<Vec<Item>> {
        let items = sqlx::query_as::<_, Item>(
            "SELECT * FROM items 
             WHERE deleted_at IS NOT NULL 
             ORDER BY deleted_at DESC"
        )
        .fetch_all(&self.pool).await?;
        Ok(items)
    }

    async fn create_item(&self, id: Uuid, title: String, motivation: i8, due: Option<DateTime<Utc>>, duration_minutes: Option<i32>) -> AppResult<()> {
        let result = sqlx::query(
            "INSERT INTO items (id, title, due, duration_minutes, status, motivation, is_archived) 
             VALUES (?, ?, ?, ?, 'todo', ?, 0)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                due = excluded.due,
                duration_minutes = excluded.duration_minutes,
                motivation = excluded.motivation,
                updated_at = CURRENT_TIMESTAMP"
        )
        .bind(id).bind(title).bind(due).bind(duration_minutes).bind(motivation)
        .execute(&self.pool).await?;

        if result.rows_affected() != 1 {
            return Err(crate::error::AppError::InvalidInput(format!(
                "Expected 1 row affected in create_item, got {}",
                result.rows_affected()
            )));
        }
        Ok(())
    }

    async fn update_item_status(&self, id: Uuid, status: TaskStatus) -> AppResult<()> {
        let result = sqlx::query(
            "UPDATE items SET status = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"
        )
            .bind(status.as_str()).bind(id).execute(&self.pool).await?;
        if result.rows_affected() != 1 {
            return Err(crate::error::AppError::InvalidInput(format!(
                "Expected 1 row affected in update_item_status for {}, got {}",
                id,
                result.rows_affected()
            )));
        }
        Ok(())
    }

    async fn update_item_details(&self, id: Uuid, title: String, description: Option<String>, due: Option<DateTime<Utc>>, duration_minutes: Option<i32>, motivation: i8) -> AppResult<()> {
        sqlx::query(
            "UPDATE items SET title = ?, description = ?, due = ?, duration_minutes = ?, motivation = ?, updated_at = CURRENT_TIMESTAMP 
             WHERE id = ?"
        )
        .bind(title).bind(description).bind(due).bind(duration_minutes).bind(motivation).bind(id)
        .execute(&self.pool).await?;
        Ok(())
    }

    async fn archive_item(&self, id: Uuid) -> AppResult<()> {
        sqlx::query(
            "UPDATE items SET is_archived = 1, updated_at = CURRENT_TIMESTAMP 
             WHERE id = ? AND deleted_at IS NULL"
        )
        .bind(id).execute(&self.pool).await?;
        Ok(())
    }

    async fn unarchive_item(&self, id: Uuid) -> AppResult<()> {
        sqlx::query(
            "UPDATE items SET is_archived = 0, updated_at = CURRENT_TIMESTAMP 
             WHERE id = ? AND deleted_at IS NULL"
        )
        .bind(id).execute(&self.pool).await?;
        Ok(())
    }

    async fn soft_delete_item(&self, id: Uuid) -> AppResult<()> {
        sqlx::query(
            "UPDATE items SET deleted_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP 
             WHERE id = ?"
        )
        .bind(id).execute(&self.pool).await?;
        Ok(())
    }

    async fn restore_item(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("UPDATE items SET deleted_at = NULL, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(id).execute(&self.pool).await?;
        Ok(())
    }

    async fn hard_delete_item(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM items WHERE id = ? AND deleted_at IS NOT NULL")
            .bind(id).execute(&self.pool).await?;
        Ok(())
    }

    async fn empty_item_trash(&self, full_wipe: bool) -> AppResult<()> {
        let sql = if full_wipe {
            "DELETE FROM items"
        } else {
            "DELETE FROM items WHERE deleted_at IS NOT NULL"
        };

        let result = sqlx::query(sql)
            .execute(&self.pool)
            .await?;

        if full_wipe {
            println!("Cleared {} total items (full wipe)", result.rows_affected());
        } else {
            println!("Cleared {} deleted items from trash", result.rows_affected());
        }
        Ok(())
    }
}