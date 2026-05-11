use axum::{Router, routing::post};
use crate::handler::auth::*;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login_hand))
        .route("/logout", post(logout_hand))
        .route("/refresh", post(refresh_hand))
}