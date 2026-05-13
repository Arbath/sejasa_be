use axum::{extract::Query, response::IntoResponse};
use http::Uri;
use uuid::Uuid;

use crate::{middleware::auth::AuthUser, models::user::{SkillQuery, UpdateProfileReq, UserQuery, UserReq, UserSkillCreate, UserSkillUpdate}, service::user::UserService, utils::{request::{ValidatedJson, ValidatedPath}, response::{ApiError, WebResponse}}};

pub async fn user_register_hand(
    uri: Uri,
    service: UserService,
    ValidatedJson(req): ValidatedJson<UserReq>
) -> Result<impl IntoResponse, ApiError> {
    let res = service.user_register(req).await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::created(&uri, "Register successfuly!", res))
}

pub async fn search_user_hand(
    Query(query): Query<UserQuery>,
    uri: Uri,
    service: UserService
) -> Result<impl IntoResponse, ApiError> {
    let res = match &query.name {
        Some(user_name) => {
            service.search_user(user_name).await
        },
        None => {
            service.find_all_users().await 
        }
    }.map_err(|e| e.with_path(&uri))?;

    let message = if let Some(n) = &query.name {
        format!("Result for '{}'", n)
    } else {
        "All users retrieved".to_string()
    };
    Ok(WebResponse::ok(&uri, &message, res))
}

pub async fn my_profile_hand(
    uri: Uri,
    AuthUser(user): AuthUser,
    service: UserService
) -> Result<impl IntoResponse, ApiError> {
    let res = service.user_profile(user.id).await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::ok(&uri, "Success", res))
}

pub async fn user_profile_hand(
    ValidatedPath(user_id): ValidatedPath<Uuid>,
    uri: Uri,
    service: UserService
) -> Result<impl IntoResponse, ApiError> {
    let res = service.user_profile(user_id).await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::ok(&uri, "Success", res))
}

pub async fn update_profile_hand(
    uri: Uri,
    AuthUser(user): AuthUser,
    service: UserService,
    ValidatedJson(data): ValidatedJson<UpdateProfileReq>
) -> Result<impl IntoResponse, ApiError> {
    let res = service.user_update(user, data).await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::ok(&uri, "Profile updated!", res))
}

pub async fn find_skills_hand(
    Query(query): Query<SkillQuery>,
    uri: Uri,
    service: UserService,
) -> Result<impl IntoResponse, ApiError> {
    let res = match &query.name {
        Some(skill_name) => {
            service.find_skills_name(skill_name).await
        },
        None => {
            service.find_all_skills().await 
        }
    }.map_err(|e| e.with_path(&uri))?;

    let message = if let Some(n) = &query.name {
        format!("Result for '{}'", n)
    } else {
        "All skills retrieved".to_string()
    };

    Ok(WebResponse::ok(&uri, &message, res))
}

pub async fn add_skills_hand(
    uri: Uri,
    AuthUser(user): AuthUser,
    service: UserService,
    ValidatedJson(data): ValidatedJson<UserSkillCreate>
) -> Result<impl IntoResponse, ApiError> {
    let res = service.add_my_skills(user, data).await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::ok(&uri, "Skills addeded!!", res))
}

pub async fn edit_skills_hand(
    ValidatedPath(skill_id): ValidatedPath<i32>,
    uri: Uri,
    AuthUser(user): AuthUser,
    service: UserService,
    ValidatedJson(data): ValidatedJson<UserSkillUpdate>
) -> Result<impl IntoResponse, ApiError> {
    let res = service.edit_my_skills(user, skill_id, data).await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::ok(&uri, "Skills updated!", res))
}

pub async fn remove_skills_hand(
    ValidatedPath(skill_id): ValidatedPath<i32>,
    uri: Uri,
    AuthUser(user): AuthUser,
    service: UserService,
) -> Result<impl IntoResponse, ApiError> {
    let res = service.remove_my_skills(user, skill_id).await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::ok(&uri, "Skills deleted!", res))
}
