use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Chat {
    pub id: Uuid,
    pub user_id: Uuid,
    pub project_id: Uuid
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ChatDetail {
    pub id: Uuid,
    pub chat_id: Uuid,
    pub content: String,
    pub file: Option<String>,
    pub is_read: bool,
    pub send_at: DateTime<Utc>
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ChatMessage {
    pub content: String,
    pub file: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "event", content = "data")]
pub enum WsEvent {
    System(String),
    History(Vec<ChatDetail>),
    NewMessage(ChatDetail),
}