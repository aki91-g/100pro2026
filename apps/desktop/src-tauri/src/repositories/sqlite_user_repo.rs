use async_trait::async_trait;
use crate::error::AppResult;
use crate::models::user::{LocalUser, LocalUserUpdate};
use crate::repositories::user_repo::UserRepository;
use sqlx::SqlitePool;

pub struct SqliteUserRepo {
    pub pool: SqlitePool,
}

#[async_trait]
impl UserRepository for SqliteUserRepo {
    async fn get_active_user(&self) -> AppResult<Option<LocalUser>> {
        let user = sqlx::query_as::<_, LocalUser>(
            "SELECT * FROM local_user WHERE is_active = 1 LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(user)
    }
    
    async fn get_user_by_id(&self, user_id: &str) -> AppResult<Option<LocalUser>> {
        let user = sqlx::query_as::<_, LocalUser>(
            "SELECT * FROM local_user WHERE id = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(user)
    }
    
    async fn upsert_user(&self, user: &LocalUserUpdate) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO local_user (id, username, last_login, is_active) 
             VALUES (?, ?, CURRENT_TIMESTAMP, 1)
             ON CONFLICT(id) DO UPDATE SET
                username = excluded.username,
                last_login = CURRENT_TIMESTAMP,
                is_active = 1"
        )
        .bind(&user.id)
        .bind(&user.username)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn update_last_login(&self, user_id: &str) -> AppResult<()> {
        sqlx::query(
            "UPDATE local_user SET last_login = CURRENT_TIMESTAMP WHERE id = ?"
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn deactivate_user(&self, user_id: &str) -> AppResult<()> {
        sqlx::query(
            "UPDATE local_user SET is_active = 0 WHERE id = ?"
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn clear_all_users(&self) -> AppResult<()> {
        sqlx::query("DELETE FROM local_user")
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
}
