use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a local user stored in SQLite
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LocalUser {
    pub id: String,  // UUID as string
    pub username: String,
    pub hashed_session: Option<String>,
    pub last_login: Option<DateTime<Utc>>,
    pub is_active: i32,  // SQLite doesn't have bool, uses 0/1
}

/// Represents a user profile in Postgres
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Profile {
    pub id: String,  // UUID as string
    pub username: Option<String>,
    pub registered_at: Option<DateTime<Utc>>,
}

/// DTO for creating/updating a local user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalUserUpdate {
    pub id: String,
    pub username: String,
}
