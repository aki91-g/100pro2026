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
    
    // Using Chrono for seamless SQL mapping
    pub due: Option<DateTime<Utc>>, 
    pub duration_minutes: Option<i32>, 
    
    pub motivation: i8, // -128 to 127
    pub is_archived: bool,
    
    // Metadata for lifecycle management
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>, // None = Active, Some = Deleted
}

impl Item {
    /// Logical check to see if the item should appear in the main UI
    pub fn is_visible(&self) -> bool {
        self.deleted_at.is_none() && !self.is_archived
    }
}