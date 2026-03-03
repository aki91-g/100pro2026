use async_trait::async_trait;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::item::{Item, TaskStatus};
use crate::error::AppResult;
use crate::repositories::item_repo::ItemRepository;
use sqlx::PgPool;

pub struct PostgresItemRepo {
    pub pool: PgPool,
}

#[async_trait]
impl ItemRepository for PostgresItemRepo {
    async fn get_active_items(&self) -> AppResult<Vec<Item>> {
        // Postgres uses NULLS FIRST/LAST syntax or standard ORDER BY
        let items = sqlx::query_as::<_, Item>(
            "SELECT * FROM items 
             WHERE deleted_at IS NULL AND is_archived = false 
             ORDER BY due ASC NULLS LAST, created_at DESC"
        )
        .fetch_all(&self.pool).await?;
        Ok(items)
    }

    async fn get_archived_items(&self) -> AppResult<Vec<Item>> {
        let items = sqlx::query_as::<_, Item>(
            "SELECT * FROM items 
             WHERE deleted_at IS NULL AND is_archived = true 
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
        // Use $1, $2 for Postgres
        sqlx::query(
            "INSERT INTO items (id, title, due, duration_minutes, status, motivation, is_archived) 
             VALUES ($1, $2, $3, $4, 'todo', $5, false)"
        )
        .bind(id).bind(title).bind(due).bind(duration_minutes).bind(motivation as i16)
        .execute(&self.pool).await?;
        Ok(())
    }

    async fn update_item_status(&self, id: Uuid, status: TaskStatus) -> AppResult<()> {
        let result = sqlx::query("UPDATE items SET status = $1 WHERE id = $2")
            .bind(status).bind(id).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(crate::error::AppError::NotFound(id.to_string()));
        }
        Ok(())
    }

    async fn update_item_details(&self, id: Uuid, title: String, description: Option<String>, due: Option<DateTime<Utc>>, duration_minutes: Option<i32>, motivation: i8) -> AppResult<()> {
        sqlx::query(
            "UPDATE items SET title = $1, description = $2, due = $3, duration_minutes = $4, motivation = $5 
             WHERE id = $6"
        )
        .bind(title).bind(description).bind(due).bind(duration_minutes).bind(motivation as i16).bind(id)
        .execute(&self.pool).await?;
        Ok(())
    }

    async fn archive_item(&self, id: Uuid) -> AppResult<()> {
        sqlx::query(
            "UPDATE items SET is_archived = true, updated_at = NOW() 
             WHERE id = $1 AND deleted_at IS NULL"
        )
        .bind(id).execute(&self.pool).await?;
        Ok(())
    }

    async fn unarchive_item(&self, id: Uuid) -> AppResult<()> {
        sqlx::query(
            "UPDATE items SET is_archived = false, updated_at = NOW() 
             WHERE id = $1 AND deleted_at IS NULL"
        )
        .bind(id).execute(&self.pool).await?;
        Ok(())
    }

    async fn soft_delete_item(&self, id: Uuid) -> AppResult<()> {
        sqlx::query(
            "UPDATE items SET deleted_at = NOW(), updated_at = NOW() 
             WHERE id = $1"
        )
        .bind(id).execute(&self.pool).await?;
        Ok(())
    }

    async fn restore_item(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("UPDATE items SET deleted_at = NULL, updated_at = NOW() WHERE id = $1")
            .bind(id).execute(&self.pool).await?;
        Ok(())
    }

    async fn hard_delete_item(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM items WHERE id = $1 AND deleted_at IS NOT NULL")
            .bind(id).execute(&self.pool).await?;
        Ok(())
    }

    async fn empty_item_trash(&self) -> AppResult<()> {
        sqlx::query("DELETE FROM items WHERE deleted_at IS NOT NULL").execute(&self.pool).await?;
        Ok(())
    }
}