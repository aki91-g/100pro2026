use async_trait::async_trait;
use sqlx::PgPool;
use crate::error::AppResult;
use crate::models::user::Profile;
use crate::repositories::profile_repo::ProfileRepository;
use uuid::Uuid;

pub struct PostgresProfileRepo {
    pub pool: PgPool,
}

#[async_trait]
impl ProfileRepository for PostgresProfileRepo {
    async fn get_profile(&self, user_id: Uuid) -> AppResult<Option<Profile>> {
        // Cast the string UUID to ::uuid for Postgres
        let profile = sqlx::query_as::<_, Profile>(
            "SELECT id::text as id, username, registered_at 
             FROM public.profiles 
             WHERE id = $1::uuid"
        )
        .bind(user_id)
        .persistent(false)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(profile)
    }
    
    async fn upsert_profile(&self, user_id: Uuid, username: &str) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO public.profiles (id, username, registered_at) 
             VALUES ($1::uuid, $2, NOW())
             ON CONFLICT (id) DO UPDATE SET
                username = EXCLUDED.username"
        )
        .bind(user_id)
        .bind(username)
        .persistent(false)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn delete_profile(&self, user_id: Uuid) -> AppResult<()> {
        sqlx::query(
            "DELETE FROM public.profiles WHERE id = $1::uuid"
        )
        .bind(user_id)
        .persistent(false)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}
