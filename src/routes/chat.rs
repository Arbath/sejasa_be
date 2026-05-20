use axum::{Router, routing::get};
use crate::{handler::chat::*, state::AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/ws/chat/{chat_id}", get(ws_chat))
        .route("/chat", get(find_chat_user_hand))
        .route("/project/{project_id}/chat", get(find_chat_project_hand))
}