use axum::{extract::Query, response::IntoResponse};
use http::Uri;
use uuid::Uuid;

use crate::{middleware::auth::{AuthAdmin, AuthUser}, models::project::{CategoryCreate, CategoryUpdate, ProjectCreate, ProjectQuery, ProjectQueryParams, ProjectUpdate}, service::project::ProjectService, utils::{request::{ValidatedJson, ValidatedPath}, response::{ApiError, WebResponse}}};


pub async fn find_nearest_project_hand(
    Query(query): Query<ProjectQuery>,
    AuthUser(user) : AuthUser,
    uri: Uri,
    service: ProjectService,
) -> Result<impl IntoResponse, ApiError> {
    let distance =  query.distance.unwrap_or(1000.0);
    let res = service.get_nearest_project(user.id, &distance).await.map_err(|e|e.with_path(&uri))?;
    let message = format!("List projects on {} meters", distance as i64);
    Ok(WebResponse::ok(&uri, &message, res))
}

pub async fn find_project_u_hand(
    uri: Uri,
    service: ProjectService,
) -> Result<impl IntoResponse, ApiError> {
    let res = service.get_all_project_unauth().await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::ok(&uri, "List projects!", res))
}

pub async fn find_one_project_u_hand(
    ValidatedPath(project_id): ValidatedPath<Uuid>,
    uri: Uri,
    service: ProjectService,
) -> Result<impl IntoResponse, ApiError> {
    let res = service.get_one_project_unauth(project_id).await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::ok(&uri, "Success!", res))
}

pub async fn find_user_project_u_hand(
    ValidatedPath(user_id): ValidatedPath<Uuid>,
    uri: Uri,
    service: ProjectService,
) -> Result<impl IntoResponse, ApiError> {
    let res = service.get_user_project_unauth(user_id).await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::ok(&uri, "Success!", res))
}

pub async fn find_project_hand(
    Query(query): Query<ProjectQueryParams>,
    uri: Uri,
    service: ProjectService,
) -> Result<impl IntoResponse, ApiError> {
    let res = service.search_all_project(query).await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::ok(&uri, "List projects!", res))
}

pub async fn find_one_project_hand(
    ValidatedPath(project_id): ValidatedPath<Uuid>,
    uri: Uri,
    AuthUser(user) : AuthUser,
    service: ProjectService,
) -> Result<impl IntoResponse, ApiError> {
    let res = service.get_one_project(project_id, user.id).await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::ok(&uri, "Success!", res))
}

pub async fn find_my_project_hand(
    uri: Uri,
    AuthUser(user) : AuthUser,
    service: ProjectService,
) -> Result<impl IntoResponse, ApiError> {
    let res = service.get_user_project(user.id).await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::ok(&uri, "Success!", res))
}

pub async fn find_user_project_hand(
    ValidatedPath(user_id): ValidatedPath<Uuid>,
    uri: Uri,
    service: ProjectService,
) -> Result<impl IntoResponse, ApiError> {
    let res = service.get_user_project(user_id).await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::ok(&uri, "Success!", res))
}


pub async fn create_project_hand(
    uri: Uri,
    AuthUser(user) : AuthUser,
    service: ProjectService,
    ValidatedJson(req): ValidatedJson<ProjectCreate>
) -> Result<impl IntoResponse, ApiError> {
    let res = service.add_project(req, user).await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::created(&uri, "Project created!", res))
}

pub async fn edit_project_hand(
    ValidatedPath(project_id): ValidatedPath<Uuid>,
    uri: Uri,
    AuthUser(user) : AuthUser,
    service: ProjectService,
    ValidatedJson(req): ValidatedJson<ProjectUpdate>
) -> Result<impl IntoResponse, ApiError> {
    let res = service.edit_project(req, project_id, user).await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::ok(&uri, "Project updated!", res))
}

pub async fn remove_project_hand(
    ValidatedPath(project_id): ValidatedPath<Uuid>,
    uri: Uri,
    AuthUser(user) : AuthUser,
    service: ProjectService,
) -> Result<impl IntoResponse, ApiError> {
    let res = service.remove_project( project_id, user).await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::ok(&uri, "Project deleted!", res))
}

pub async fn find_category_hand(
    uri: Uri,
    service: ProjectService,
) -> Result<impl IntoResponse, ApiError> {
    let res = service.get_all_category().await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::ok(&uri, "List category!", res))
}

pub async fn find_one_category_hand(
    ValidatedPath(category_name): ValidatedPath<String>,
    uri: Uri,
    service: ProjectService,
) -> Result<impl IntoResponse, ApiError> {
    let res = service.get_one_category(category_name).await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::ok(&uri, "Success!", res))
}

pub async fn create_category_hand(
    uri: Uri,
    AuthAdmin(user) : AuthAdmin,
    service: ProjectService,
    ValidatedJson(req): ValidatedJson<CategoryCreate>
) -> Result<impl IntoResponse, ApiError> {
    let res = service.add_category(req, user).await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::created(&uri, "Category created!", res))
}

pub async fn edit_category_hand(
    ValidatedPath(category_id): ValidatedPath<i32>,
    uri: Uri,
    AuthAdmin(user) : AuthAdmin,
    service: ProjectService,
    ValidatedJson(req): ValidatedJson<CategoryUpdate>
) -> Result<impl IntoResponse, ApiError> {
    let res = service.edit_category(req, category_id, user).await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::ok(&uri, "Category updated!", res))
}

pub async fn remove_category_hand(
    ValidatedPath(category_id): ValidatedPath<i32>,
    uri: Uri,
    AuthAdmin(user) : AuthAdmin,
    service: ProjectService,
) -> Result<impl IntoResponse, ApiError> {
    let res = service.remove_category( category_id, user).await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::ok(&uri, "Category deleted!", res))
}