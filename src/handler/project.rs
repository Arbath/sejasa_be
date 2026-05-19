use axum::{extract::Query, response::IntoResponse};
use http::Uri;
use uuid::Uuid;

use crate::{middleware::auth::{AuthAdmin, AuthUser}, models::{project::{CategoryCreate, CategoryUpdate, ProjectCreate, ProjectParticipantUpdate, ProjectQueryParams, ProjectUpdate, ProjectUserQuery}, review::ReviewReq}, service::project::ProjectService, utils::{request::{ValidatedJson, ValidatedPath}, response::{ApiError, PaginationMeta, WebResponse}}};

pub async fn find_project_hand(
    Query(query): Query<ProjectQueryParams>,
    uri: Uri,
    service: ProjectService,
) -> Result<impl IntoResponse, ApiError> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);
    let (data, total_items) = service.search_all_project(query).await.map_err(|e|e.with_path(&uri))?;
    let total_pages = (total_items as f64 / limit as f64).ceil() as i64;
    let meta = PaginationMeta {
        current_page: page,
        limit_page: limit,
        total_items,
        total_pages
    };
    Ok(WebResponse::ok_paginated(&uri, "List projects!", data, meta))
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
    Query(query): Query<ProjectUserQuery>,
    uri: Uri,
    service: ProjectService,
) -> Result<impl IntoResponse, ApiError> {
    let res = match query.sort.as_deref() {
        Some("uploaded") => service.get_user_project_uploaded(user_id).await,
        Some("pending") => service.get_user_project_applyed(user_id, "pending".to_string()).await,
        Some("accepted") => service.get_user_project_applyed(user_id, "accepted".to_string()).await,
        Some("rejected") => service.get_user_project_applyed(user_id, "rejected".to_string()).await,
        _ => service.get_user_project(user_id).await,
    }
    .map_err(|e| e.with_path(&uri))?;

    let message = match query.sort.as_deref() {
        Some(status) => format!("Result for '{status}'"),
        None => "All project retrieved".to_string(),
    };

    Ok(WebResponse::ok(&uri, &message, res))
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

pub async fn add_project_hastag_hand(
    ValidatedPath((project_id, hastag_name)): ValidatedPath<(Uuid, String)>,
    uri: Uri,
    AuthUser(user) : AuthUser,
    service: ProjectService,
) -> Result<impl IntoResponse, ApiError> {
    let res = service.add_hastags( project_id, &hastag_name, user).await.map_err(|e|e.with_path(&uri))?;
    let message = format!("Hastag '{}' addeded!", hastag_name);
    Ok(WebResponse::ok(&uri, &message, res))
}

pub async fn remove_project_hastag_hand(
    ValidatedPath((project_id, hastag_name)): ValidatedPath<(Uuid, String)>,
    uri: Uri,
    AuthUser(user) : AuthUser,
    service: ProjectService,
) -> Result<impl IntoResponse, ApiError> {
    let res = service.remove_hastags( project_id, &hastag_name, user).await.map_err(|e|e.with_path(&uri))?;
    let message = format!("Hastag '{}' deleted!", hastag_name);
    Ok(WebResponse::ok(&uri, &message, res))
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

pub async fn apply_project_hand(
    ValidatedPath(project_id): ValidatedPath<Uuid>,
    uri: Uri,
    AuthUser(user) : AuthUser,
    service: ProjectService,
) -> Result<impl IntoResponse, ApiError> {
    let res = service.apply_project( project_id, user).await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::ok(&uri, "Project applyed!", res))
}

pub async fn list_participant_hand(
    ValidatedPath(project_id): ValidatedPath<Uuid>,
    uri: Uri,
    AuthUser(_) : AuthUser,
    service: ProjectService,
) -> Result<impl IntoResponse, ApiError> {
    let res = service.list_participant_project(project_id).await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::ok(&uri, "List project participant", res))
}

pub async fn apply_participant_hand(
    ValidatedPath((project_id, project_part_id)): ValidatedPath<(Uuid, i32)>,
    uri: Uri,
    AuthUser(user) : AuthUser,
    service: ProjectService,
    ValidatedJson(req): ValidatedJson<ProjectParticipantUpdate>
) -> Result<impl IntoResponse, ApiError> {
    let res = service.apply_participant_project( project_id, project_part_id, user, req).await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::ok(&uri, "Participant updated!", res))
}

pub async fn review_participant_hand(
    ValidatedPath((project_id, participant_id)): ValidatedPath<(Uuid, Uuid)>,
    uri: Uri,
    AuthUser(user) : AuthUser,
    service: ProjectService,
    ValidatedJson(req): ValidatedJson<ReviewReq>
) -> Result<impl IntoResponse, ApiError> {
    let res = service.review_participant( project_id, participant_id, req, user).await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::ok(&uri, "Participant reviewed!", res))
}

pub async fn review_all_participant_hand(
    ValidatedPath(project_id): ValidatedPath<Uuid>,
    uri: Uri,
    AuthUser(user) : AuthUser,
    service: ProjectService,
    ValidatedJson(req): ValidatedJson<ReviewReq>
) -> Result<impl IntoResponse, ApiError> {
    let res = service.review_all_participant( project_id, req, user).await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::ok(&uri, "All participant reviewed!", res))
}

pub async fn review_project_hand(
    ValidatedPath(project_id): ValidatedPath<Uuid>,
    uri: Uri,
    AuthUser(user) : AuthUser,
    service: ProjectService,
    ValidatedJson(req): ValidatedJson<ReviewReq>
) -> Result<impl IntoResponse, ApiError> {
    let res = service.review_project( project_id, req, user).await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::ok(&uri, "Project reviewed!", res))
}