use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LocalSession {
    pub id: i32, // Always 1
    #[sqlx(try_from = "String")]
    pub user_id: Uuid,
    pub username: String,
    pub access_token: Option<String>,
    pub last_login: Option<String>,
}
