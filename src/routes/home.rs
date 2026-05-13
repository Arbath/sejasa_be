use axum::{Router, routing::get};
use crate::{handler::chat::{test, ws_hand, ws_chat}, state::AppState};

async fn home() -> &'static str {
    "Sejasa rust backend is running..."
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(home))
        .route("/user", get(test))
        .route("/ws", get(ws_hand))
        .route("/ws/chat", get(ws_chat))
}