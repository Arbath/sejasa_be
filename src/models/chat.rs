use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, types::Json};
use uuid::Uuid;

use crate::models::user::UserPrev;

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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ChatPreview {
    pub id: Uuid,
    pub project_id: Uuid,
    pub user: Json<UserPrev>,
    pub title: String,
    pub body: String,
    pub unread_msg: i32,
    pub timestamp: Option<DateTime<Utc>>
}

#[derive(Debug, Serialize)]
#[serde(tag = "event", content = "data")]
pub enum WsEvent {
    System(String),
    History(Vec<ChatDetail>),
    NewMessage(ChatDetail),
}