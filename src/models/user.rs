use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
pub enum AccountType {
    PERSONAL,
    ORGANIZATION
}
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
pub enum Gender {
    MALE,
    FEMALE,
    UNGENDER
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid, 
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub is_superuser: bool,
    pub created_at: DateTime<Utc>,
    pub account_type: AccountType,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserProfile {
    pub id: Uuid, 
    pub name: String,
    pub email: String,
    pub gender: Option<String>,
    pub descriptions: Option<String>, 
    pub rating: Option<f64>,
    pub contact: Option<String>, 
    pub address: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub image: Option<String>,
    pub account_type: AccountType,
    pub created_at: DateTime<Utc>
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserProfileRes {
    pub id: Uuid, 
    pub name: String,
    pub email: String,
    pub gender: Option<String>,
    pub descriptions: Option<String>, 
    pub rating: Option<f64>,
    pub contact: Option<String>,
    pub skills: Vec<UserSkill>,
    pub address: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub image: Option<String>,
    pub account_type: AccountType,
    pub created_at: DateTime<Utc>
}

impl UserProfileRes {
    pub fn combine(profile: UserProfile, skills: Vec<UserSkill>) -> Self {
        let UserProfile { 
            id, name, email, gender,descriptions, rating, contact, 
            address, latitude, longitude, image, account_type, created_at 
        } = profile;

        Self { 
            id, 
            name, 
            email, 
            gender, 
            descriptions,
            rating, 
            contact, 
            skills,
            address, 
            latitude, 
            longitude, 
            image, 
            account_type, 
            created_at 
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UpdateProfileReq {
    pub name: Option<String>,
    pub email: Option<String>,
    pub gender: Option<String>,
    pub descriptions: Option<String>, 
    pub contact: Option<String>, 
    pub address: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserReq {
    pub name: String,
    pub password1: String,
    pub password2: String,
    pub email: String,
    pub gender: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub account_type: AccountType,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserCreate {
    pub name: String,
    pub password: String,
    pub email: String,
    pub gender: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub account_type: AccountType,
}

impl UserCreate {
    pub fn from_req(req: UserReq, password_hash: String) -> Self {

        Self { 
            name: req.name, 
            password: password_hash, 
            email: req.email, 
            gender: req.gender,
            latitude: req.latitude, 
            longitude: req.longitude, 
            account_type: req.account_type
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct  UserSkill {
    pub id: i32,
    pub name: String,
    pub descriptions: Option<String>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct  UserSkillCreate {
    pub name: String,
    pub descriptions: Option<String>
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct  UserSkillUpdate {
    pub name: String,
    pub descriptions: Option<String>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct  SkillQuery {
    pub name: Option<String>
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct  UserQuery {
    pub name: Option<String>
}