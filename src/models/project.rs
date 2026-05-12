use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use serde_json::Value;
use uuid::Uuid;

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
    pub user_id: Option<Uuid>, 
    pub name: String,
    pub address: Option<String>,
    pub max_participant: Option<i32>,
    pub status: ProjectStatus,
    pub descriptions: Option<String>,
    pub requirements: Option<Value>,
    pub slug: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub rating: Option<f64>,
    pub updated_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub category_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectRes {
    pub id: Uuid,
    pub name: String,
    pub address: Option<String>,
    pub participant: String,
    pub status: ProjectStatus,
    pub descriptions: Option<String>,
    pub requirements: Option<Value>,
    pub distance: String,
    pub rating: Option<f64>,
    pub category: String,
    pub slug: Option<String>,
    pub owner: ProjectOwner,
    pub updated_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectOwner {
    pub id: Uuid,
    pub name: String,
    pub rating: f64,
    pub image: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCreate {
    pub name: String,
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
pub struct Hastag {
    pub id: i32,
    pub project_id: Option<Uuid>,
    pub name: String,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HastagCreate {
    pub project_id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HastagUpdate {
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