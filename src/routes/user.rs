use axum::{Router, routing::{get, patch, post}};
use crate::handler::user::*;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(user_register_hand))
        .route("/user/{user_id}", get(user_profile_hand))
        .route("/me", get(my_profile_hand))
        .route("/me", patch(update_profile_hand))
}