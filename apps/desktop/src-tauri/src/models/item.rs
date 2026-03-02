use uuid::Uuid;
use chrono::{DateTime, Utc}; // Recommended for SQLX compatibility
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "status", rename_all = "lowercase")]
pub enum TaskStatus {
    Backlog,
    Todo,
    InProgress,
    Done,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Item {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    
    pub due: Option<DateTime<Utc>>, 
    pub duration_minutes: Option<i32>, 
    
    pub motivation: i32, 
    
    // SQLite uses 0/1 for booleans. 
    // We tell sqlx to map the SQLite Integer to a Rust bool.
    pub is_archived: bool,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Item {
    /// Logical check to see if the item should appear in the main UI
    pub fn is_visible(&self) -> bool {
        self.deleted_at.is_none() && !self.is_archived
    }
}