use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Review {
    pub id: i32,
    pub user_id: Uuid,
    pub project_id: Uuid,
    pub reviewer_id: Uuid,
    pub rating: f64,
    pub review: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReviewReq {
    pub rating: f64,
    pub review: String,
}