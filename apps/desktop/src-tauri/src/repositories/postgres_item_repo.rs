use async_trait::async_trait;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::item::{Item, TaskStatus};
use crate::error::AppResult;
use crate::repositories::item_repo::ItemRepository;
use sqlx::PgPool;
use tracing::{warn};

pub struct PostgresItemRepo {
    pub pool: PgPool,
}

#[async_trait]
impl ItemRepository for PostgresItemRepo {
    async fn get_all_items(&self, user_id: &str) -> AppResult<Vec<Item>> {
        let items = sqlx::query_as::<_, Item>(
            "SELECT * FROM items WHERE user_id = $1 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .persistent(false)
        .fetch_all(&self.pool).await?;
        Ok(items)
    }

    async fn get_active_items(&self, user_id: &str) -> AppResult<Vec<Item>> {
        // Postgres uses NULLS FIRST/LAST syntax or standard ORDER BY
        let items = sqlx::query_as::<_, Item>(
            "SELECT * FROM items 
             WHERE user_id = $1 AND deleted_at IS NULL AND is_archived = false 
             ORDER BY due ASC NULLS LAST, created_at DESC"
        )
        .bind(user_id)
        .persistent(false)
        .fetch_all(&self.pool).await?;
        Ok(items)
    }

    async fn get_archived_items(&self, user_id: &str) -> AppResult<Vec<Item>> {
        let items = sqlx::query_as::<_, Item>(
            "SELECT * FROM items 
             WHERE user_id = $1 AND deleted_at IS NULL AND is_archived = true 
             ORDER BY created_at DESC"
        )
        .bind(user_id)
        .persistent(false)
        .fetch_all(&self.pool).await?;
        Ok(items)
    }

    async fn get_deleted_items(&self, user_id: &str) -> AppResult<Vec<Item>> {
        let items = sqlx::query_as::<_, Item>(
            "SELECT * FROM items 
             WHERE user_id = $1 AND deleted_at IS NOT NULL 
             ORDER BY deleted_at DESC"
        )
        .bind(user_id)
        .persistent(false)
        .fetch_all(&self.pool).await?;
        Ok(items)
    }

    async fn create_item(&self, user_id: &str, id: Uuid, title: String, motivation: i8, due: Option<DateTime<Utc>>, duration_minutes: Option<i32>) -> AppResult<()> {
        let result = sqlx::query(
            r#"INSERT INTO items (id, user_id, title, due, duration_minutes, status, motivation, is_archived) 
            VALUES ($1, $2, $3, $4, $5, 'todo', $6, false)
            ON CONFLICT (id) DO UPDATE SET
                title = EXCLUDED.title,
                due = EXCLUDED.due,
                duration_minutes = EXCLUDED.duration_minutes,
                motivation = EXCLUDED.motivation,
                updated_at = NOW()"#
        )
        .persistent(false)
        .bind(id).bind(user_id).bind(title).bind(due).bind(duration_minutes).bind(motivation as i16)
        .execute(&self.pool).await?;

        if result.rows_affected() != 1 {
            return Err(crate::error::AppError::InvalidInput(format!(
                "Expected 1 row affected in create_item, got {}",
                result.rows_affected()
            )));
        }
        Ok(())
    }

    async fn update_item_status(&self, user_id: &str, id: Uuid, status: TaskStatus) -> AppResult<()> {
        let result = sqlx::query(
            "UPDATE items SET status = $1, updated_at = NOW() WHERE id = $2 AND user_id = $3"
        )
        .persistent(false)
        .bind(status.as_str())
        .bind(id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() != 1 {
            warn!("Unexpected status update affected rows for item {}: {}", id, result.rows_affected());
            return Err(crate::error::AppError::InvalidInput(format!(
                "Expected 1 row affected in update_item_status for {}, got {}",
                id,
                result.rows_affected()
            )));
        }

        Ok(())
    }

    async fn update_item_details(&self, user_id: &str, id: Uuid, title: String, description: Option<String>, due: Option<DateTime<Utc>>, duration_minutes: Option<i32>, motivation: i8) -> AppResult<()> {
        let result = sqlx::query(
            "UPDATE items SET title = $1, description = $2, due = $3, duration_minutes = $4, motivation = $5 
             WHERE id = $6 AND user_id = $7"
        )
        .persistent(false)
        .bind(title).bind(description).bind(due).bind(duration_minutes).bind(motivation as i16).bind(id).bind(user_id)
        .execute(&self.pool).await?;
        if result.rows_affected() == 0 {
             return Err(crate::error::AppError::NotFound(id.to_string()));
        }
        Ok(())
    }

    async fn archive_item(&self, user_id: &str, id: Uuid) -> AppResult<()> {
        sqlx::query(
            "UPDATE items SET is_archived = true, updated_at = NOW() 
             WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL"
        )
        .persistent(false)
        .bind(id).bind(user_id).execute(&self.pool).await?;
        Ok(())
    }

    async fn unarchive_item(&self, user_id: &str, id: Uuid) -> AppResult<()> {
        sqlx::query(
            "UPDATE items SET is_archived = false, updated_at = NOW() 
             WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL"
        )
        .persistent(false)
        .bind(id).bind(user_id).execute(&self.pool).await?;
        Ok(())
    }

    async fn soft_delete_item(&self, user_id: &str, id: Uuid) -> AppResult<()> {
        sqlx::query(
            "UPDATE items SET deleted_at = NOW(), updated_at = NOW() 
             WHERE id = $1 AND user_id = $2"
        )
        .persistent(false)
        .bind(id).bind(user_id).execute(&self.pool).await?;
        Ok(())
    }

    async fn restore_item(&self, user_id: &str, id: Uuid) -> AppResult<()> {
        sqlx::query("UPDATE items SET deleted_at = NULL, updated_at = NOW() WHERE id = $1 AND user_id = $2")
            .persistent(false)
            .bind(id).bind(user_id).execute(&self.pool).await?;
        Ok(())
    }

    async fn hard_delete_item(&self, user_id: &str, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM items WHERE id = $1 AND user_id = $2 AND deleted_at IS NOT NULL")
            .persistent(false)
            .bind(id).bind(user_id).execute(&self.pool).await?;
        Ok(())
    }

    async fn empty_item_trash(&self, user_id: &str, full_wipe: bool) -> AppResult<()> {
        let sql = if full_wipe {
            "DELETE FROM items WHERE user_id = $1"
        } else {
            "DELETE FROM items WHERE user_id = $1 AND deleted_at IS NOT NULL"
        };

        sqlx::query(sql)
            .persistent(false)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn claim_offline_items(&self, user_id: &str) -> AppResult<usize> {
        let result = sqlx::query(
            "UPDATE items SET user_id = $1, updated_at = NOW() WHERE user_id IS NULL"
        )
        .persistent(false)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        let claimed_count = result.rows_affected() as usize;
        if claimed_count > 0 {
            println!("Claimed {} offline items for user: {}", claimed_count, user_id);
        }
        Ok(claimed_count)
    }
}