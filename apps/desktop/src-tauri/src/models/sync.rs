#[derive(serde::Serialize, Clone)]
pub struct SyncEvent {
    pub id: uuid::Uuid,
    pub status: String, // "pending", "success", "error"
    pub message: Option<String>,
}