use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use serde_json::Value;
use uuid::Uuid;

use crate::models::user::UserProfile;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
pub enum ProjectStatus {
    DONE,
    HIRING,
    PENDING,
    CANCEL
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
pub enum ProjectParticipantStatus {
    ACCEPTED,
    PENDING,
    REJECTED
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
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
    pub address: String,
    pub participant: String,
    pub status: ProjectStatus,
    pub descriptions: String,
    pub requirements: Value,
    pub distance: i32,
    pub rating: Option<f64>,
    pub category: String,
    pub slug: String,
    pub owner: ProjectOwner,
    pub updated_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

impl ProjectRes {
    pub fn into_model(project: Project, owner: UserProfile, cur_participant: i32, distance: i32, category_name: String) -> Self {
        let project_owner = ProjectOwner { 
            id: owner.id, name: owner.name, rating: owner.rating.unwrap_or(0.0), image: owner.image 
        };
        let participant = format!("{}/{}", cur_participant, project.max_participant);

        Self { 
            id: project.id, 
            name: project.name, 
            address: project.address, 
            participant, 
            status: project.status, 
            descriptions: project.descriptions, 
            requirements: project.requirements, 
            distance, 
            rating: project.rating, 
            category: category_name, 
            slug: project.slug, 
            owner: project_owner, 
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
    pub status: ProjectStatus,
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
    pub user_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectParticipantCreate {
    pub user_id: Uuid,
    pub project_id: Uuid,
    pub status: ProjectParticipantStatus
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectParticipantUpdate {
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct  ProjectQuery {
    pub distance: Option<f64>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectQueryParams {
    pub q: Option<String>,          // ?q=pembuatan+website+sekolah
    pub sort: Option<String>,       // ?sort=newest atau ?sort=closest
    pub status: Option<String>,     // ?status=done
    pub category: Option<String>,   // ?category=webdev,sekolah,programmer
    pub distance: Option<f64>,      // ?distance=10000
    pub lat: Option<f64>,           // Harus dikirim oleh frontend jika mau filter/sort by distance
    pub lon: Option<f64>,           // Harus dikirim oleh frontend jika mau filter/sort by distance
}