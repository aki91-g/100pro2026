use async_trait::async_trait;
use crate::error::AppResult;
use crate::models::session::LocalSession;
use crate::repositories::session_repo::SessionRepository;
use sqlx::SqlitePool;

pub struct SqliteSessionRepo {
    pub pool: SqlitePool,
}

#[async_trait]
impl SessionRepository for SqliteSessionRepo {
    async fn get_active_session(&self) -> AppResult<Option<LocalSession>> {
        let session = sqlx::query_as::<_, LocalSession>(
            "SELECT id, CAST(user_id AS TEXT) AS user_id, username, access_token, 
                    datetime(last_login) as last_login 
             FROM local_session 
             WHERE id = 1 
             LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(session)
    }
    
    async fn save_session(&self, user_id: &uuid::Uuid, username: &str, access_token: Option<&str>) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO local_session (id, user_id, username, access_token, last_login) 
             VALUES (1, ?, ?, ?, CURRENT_TIMESTAMP)
             ON CONFLICT(id) DO UPDATE SET
                 user_id = excluded.user_id,
                 username = excluded.username,
                 access_token = excluded.access_token,
                 last_login = CURRENT_TIMESTAMP"
        )
        .bind(user_id.to_string())
        .bind(username)
        .bind(access_token)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn clear_session(&self) -> AppResult<()> {
        sqlx::query("DELETE FROM local_session WHERE id = 1")
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    async fn update_last_login(&self) -> AppResult<()> {
        sqlx::query(
            "UPDATE local_session SET last_login = CURRENT_TIMESTAMP WHERE id = 1"
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}
