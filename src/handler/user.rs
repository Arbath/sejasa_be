use axum::{response::IntoResponse};
use http::Uri;
use uuid::Uuid;

use crate::{middleware::auth::AuthUser, models::user::{UpdateProfileReq, UserReq}, service::user::UserService, utils::{request::{ValidatedJson, ValidatedPath}, response::{ApiError, WebResponse}}};

pub async fn user_register_hand(
    uri: Uri,
    service: UserService,
    ValidatedJson(req): ValidatedJson<UserReq>
) -> Result<impl IntoResponse, ApiError> {
    let res = service.user_register(req).await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::created(&uri, "Register successfuly!", res))
}

pub async fn my_profile_hand(
    uri: Uri,
    AuthUser(user): AuthUser,
    service: UserService
) -> Result<impl IntoResponse, ApiError> {
    let res = service.user_profile(user.id).await.map_err(|e|e.with_path(&uri))?;
    println!("{:?}", user);
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
