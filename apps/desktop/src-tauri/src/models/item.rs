use uuid::Uuid;
use chrono::{DateTime, Utc}; // Recommended for SQLX compatibility
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskStatus {
    Backlog,
    Todo,
    InProgress,
    Done,
}

impl TaskStatus {
    /// Convert to lowercase string for database storage
    pub fn as_str(&self) -> &str {
        match self {
            TaskStatus::Backlog => "backlog",
            TaskStatus::Todo => "todo",
            TaskStatus::InProgress => "inprogress",
            TaskStatus::Done => "done",
        }
    }
}

impl Serialize for TaskStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for TaskStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self::from(s))
    }
}

// Implement sqlx::Type, Encode, and Decode for both Postgres and Sqlite
impl sqlx::Type<sqlx::Postgres> for TaskStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for TaskStatus {
    fn encode_by_ref(&self, args: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <String as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&self.as_str().to_string(), args)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for TaskStatus {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        Ok(Self::from(s))
    }
}

impl sqlx::Type<sqlx::Sqlite> for TaskStatus {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl sqlx::Encode<'_, sqlx::Sqlite> for TaskStatus {
    fn encode_by_ref(&self, args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'_>>) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <String as sqlx::Encode<sqlx::Sqlite>>::encode_by_ref(&self.as_str().to_string(), args)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for TaskStatus {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        Ok(Self::from(s))
    }
}
impl From<String> for TaskStatus {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "todo" => Self::Todo,
            "inprogress" => Self::InProgress,
            "done" => Self::Done,
            _ => Self::Backlog,
        }
    }
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
    pub user_id: Option<String>,
}

impl Item {
    /// Logical check to see if the item should appear in the main UI
    pub fn is_visible(&self) -> bool {
        self.deleted_at.is_none() && !self.is_archived
    }
}