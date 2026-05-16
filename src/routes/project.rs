use axum::{Router, routing::{delete, get, patch, post}};
use crate::handler::project::*;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/project/user/{user_id}/unauth", get(find_user_project_u_hand))
        .route("/project/unauth", get(find_project_u_hand))
        .route("/project/{project_id}/unauth", get(find_one_project_u_hand))
        
        .route("/project/user/{user_id}", get(find_user_project_hand))
        .route("/project/nearest", get(find_nearest_project_hand))
        .route("/project/user", get(find_my_project_hand))

        .route("/project", get(find_project_hand))
        .route("/project", post(create_project_hand))
        .route("/project/{project_id}", get(find_one_project_hand))
        .route("/project/{project_id}", patch(edit_project_hand))
        .route("/project/{project_id}", delete(remove_project_hand))
        
        .route("/project/{project_id}/hastag/{hastag_name}", post(add_project_hastag_hand))
        .route("/project/{project_id}/hastag/{hastag_name}", delete(remove_project_hastag_hand))
        
        .route("/project/category", get(find_category_hand))
        .route("/project/category", post(create_category_hand))
        .route("/project/category/name/{category_name}", get(find_one_category_hand))
        .route("/project/category/{category_id}", patch(edit_category_hand))
        .route("/project/category/{category_id}", delete(remove_category_hand))
}