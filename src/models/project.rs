use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use serde_json::Value;
use uuid::Uuid;
use sqlx::types::Json;

use crate::models::user::UserProfile;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
pub enum ProjectStatus {
    DONE,
    HIRING,
    PENDING,
    CANCEL,
    ONGOING
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
pub enum ProjectParticipantStatus {
    ACCEPTED,
    PENDING,
    REJECTED
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub descriptions: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CategoryCreate {
    pub name: String,
    pub descriptions: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CategoryUpdate {
    pub name: Option<String>,
    pub descriptions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub status: ProjectStatus,
    pub rating: Option<f64>,
    pub descriptions: String,
    pub requirements: Value,
    pub slug: String,
    pub address: String,
    pub latitude: f64,
    pub longitude: f64,
    pub max_participant: i32,
    #[sqlx(default)]
    pub cur_participant: Option<i64>,
    #[sqlx(default)]
    pub hastags: Vec<String>,
    #[sqlx(default)]
    pub distance_meters: Option<f64>,
    #[sqlx(default)]
    pub category_name: String,
    #[sqlx(default)]
    pub owner_name: String,
    #[sqlx(default)]
    pub owner_rating: f64,
    #[sqlx(default)]
    pub owner_image: Option<String>,
    pub participant_status: ProjectParticipantStatus,
    pub updated_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

impl Project {
    pub fn into_model(self, hastags: Vec<String>) -> Self {
        Self {hastags, ..self}
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectRes {
    pub id: Uuid,
    pub name: String,
    pub status: ProjectStatus,
    #[sqlx(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participant_status: Option<ProjectParticipantStatus>,
    pub rating: Option<f64>,
    pub descriptions: String,
    pub slug: String,
    pub address: String,
    pub latitude: f64,
    pub longitude: f64,
    #[sqlx(default)]
    pub distance_meters: Option<f64>,
    pub max_participant: i32,
    #[sqlx(default)]
    pub cur_participant: Option<i64>,
    pub requirements: Value,
    #[sqlx(default)]
    pub hastags: Vec<String>,
    pub category: Json<Category>,
    pub owner: Json<ProjectOwner>,
    pub updated_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

impl ProjectRes {
    pub fn into_model(project: Project, owner: UserProfile, category: Category) -> Self {
        let project_owner = ProjectOwner { 
            id: owner.id, name: owner.name, rating: owner.rating.unwrap_or(0.0), image: owner.image 
        };

        Self { 
            id: project.id, 
            name: project.name, 
            address: project.address, 
            status: project.status, 
            descriptions: project.descriptions, 
            requirements: project.requirements, 
            distance_meters: project.distance_meters, 
            rating: project.rating, 
            slug: project.slug,
            latitude: project.latitude,
            longitude: project.longitude,
            max_participant: project.max_participant,
            cur_participant: project.cur_participant,
            participant_status: Some(project.participant_status),
            hastags: project.hastags,
            category: Json(category), 
            owner: Json(project_owner), 
            updated_at: project.updated_at, 
            created_at: project.created_at 
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectOwner {
    pub id: Uuid,
    pub name: String,
    pub rating: f64,
    pub image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCreate {
    pub name: String,
    pub address: String,
    pub max_participant: i32,
    pub status: ProjectStatus,
    pub descriptions: String,
    pub requirements: Value,
    pub slug: String,
    pub latitude: f64,
    pub longitude: f64,
    pub hastags: Option<Vec<String>>,
    pub category_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectUpdate {
    pub name: Option<String>,
    pub address: Option<String>,
    pub max_participant: Option<i32>,
    pub status: Option<ProjectStatus>,
    pub descriptions: Option<String>,
    pub requirements: Option<Value>,
    pub slug: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub category_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Hastags {
    pub id: i32,
    pub project_id: Option<Uuid>,
    pub name: String,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HastagsCreate {
    pub project_id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HastagsUpdate {
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectParticipant {
    pub id: i32,
    pub user_id: Uuid,
    pub project_id: Uuid,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectParticipantCreate {
    pub user_id: Uuid,
    pub project_id: Uuid,
    pub status: ProjectParticipantStatus
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectParticipantUpdate {
    pub status: ProjectParticipantStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct  ProjectQuery {
    pub distance: Option<f64>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct  ProjectUserQuery {
    pub sort: Option<String>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectQueryParams {
    pub q: Option<String>,          // ?q=pembuatan+website+sekolah
    pub sort: Option<String>,       // ?sort=newest atau ?sort=closest
    pub status: Option<String>,     // ?status=done
    pub category: Option<String>,   // ?category=webdev,sekolah,programmer
    pub distance: Option<f64>,      // ?distance=10000
    pub most_distance: Option<String>, // ?most_distance=nearest/farthest
    pub lat: Option<f64>,           // Harus dikirim oleh frontend jika mau filter/sort by distance
    pub lon: Option<f64>,           // Harus dikirim oleh frontend jika mau filter/sort by distance
    pub page: Option<i64>,          // page yang dibaca
    pub limit: Option<i64>,         // limit per page
}

#[derive(FromRow)]
pub struct ProjectPaginationRow {
    #[sqlx(flatten)]
    pub project: ProjectRes,
    pub total_items: i64, // Hasil dari COUNT(*) OVER()
}