use axum::{Router, routing::{delete, get, patch, post}};
use crate::handler::user::*;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(user_register_hand))
        .route("/user/{user_id}", get(user_profile_hand))
        .route("/user", get(search_user_hand))
        .route("/me", get(my_profile_hand))
        .route("/me", patch(update_profile_hand))
        .route("/user/skills", get(find_skills_hand))
        .route("/user/skills", post(add_skills_hand))
        .route("/user/skills/{skill_id}", patch(edit_skills_hand))
        .route("/user/skills/{skill_id}", delete(remove_skills_hand))
}