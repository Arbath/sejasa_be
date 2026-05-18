use sqlx::PgPool;
use uuid::Uuid;

use crate::models::review::*;

pub struct ReviewRepository {
    pool: PgPool
}

impl ReviewRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_all_by_project(&self, project_id: Uuid) -> Result<Vec<Review>, sqlx::Error> {
        sqlx::query_as::<_, Review> (
            r#"SELECT * FROM review WHERE project_id = $1"#
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
    }
    
    pub async fn find_all_by_user(&self, user_id: Uuid) -> Result<Vec<Review>, sqlx::Error> {
        sqlx::query_as::<_, Review> (
            r#"SELECT * FROM review WHERE user_id = $1"#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }
    
    pub async fn find_one(&self, review_id: Uuid) -> Result<Review, sqlx::Error> {
        sqlx::query_as::<_, Review> (
            r#"SELECT * FROM review WHERE id = $1"#
        )
        .bind(review_id)
        .fetch_one(&self.pool)
        .await
    }
    
    pub async fn create(&self, user_id: Uuid, project_id: Uuid, reviewer_id: Uuid, data: ReviewReq) -> Result<Review, sqlx::Error> {
        sqlx::query_as::<_, Review> (
            r#"INSERT INTO review(user_id, project_id, reviewer_id, rating, review) VALUES($1, $2, $3, $4, $5) RETURNING *"#
        )
        .bind(user_id)
        .bind(project_id)
        .bind(reviewer_id)
        .bind(data.rating)
        .bind(data.review)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn create_all_participant(&self, project_id: Uuid, reviewer_id: Uuid, data: ReviewReq) -> Result<Vec<Review>, sqlx::Error> {
        let inserted_reviews = sqlx::query_as::<_, Review>(
            r#"
            INSERT INTO review (user_id, project_id, reviewer_id, rating, review)
            SELECT 
                user_id, project_id, $1, $2, $3
            FROM project_participant
            WHERE project_id = $4 AND status = 'accepted'
            AND NOT EXISTS (
                SELECT 1 FROM review r 
                WHERE r.project_id = project_participant.project_id 
                  AND r.user_id = project_participant.user_id
            )
            RETURNING *
            "#
        )
        .bind(reviewer_id)     
        .bind(data.rating)     
        .bind(data.review)     
        .bind(project_id)      
        .fetch_all(&self.pool) 
        .await?;

        if inserted_reviews.is_empty() {
            return Ok(inserted_reviews);
        }

        let user_ids: Vec<Uuid> = inserted_reviews.iter().map(|r| r.user_id).collect();

        sqlx::query(
            r#"
            UPDATE user_profile up
            SET rating = (
                SELECT COALESCE(AVG(gabungan.rating), 0.0)
                FROM (
                    SELECT rating FROM review WHERE user_id = up.user_id
                    UNION ALL
                    SELECT r.rating FROM review r
                    JOIN projects p ON r.project_id = p.id
                    WHERE p.user_id = up.user_id
                ) AS gabungan
            )
            -- JURUS DEWA: Cocokkan id dengan array yang dikirim dari Rust!
            WHERE up.user_id = ANY($1)
            "#
        )
        .bind(&user_ids)
        .execute(&self.pool)
        .await?;

        Ok(inserted_reviews)
    }

    pub async fn check_all_accepted_participants_reviewed(&self, project_id: Uuid) -> Result<bool, sqlx::Error> {
        let all_reviewed: bool = sqlx::query_scalar(
            r#"
            SELECT (
                EXISTS (SELECT 1 FROM project_participant WHERE project_id = $1 AND status = 'accepted')
                AND
                NOT EXISTS (
                    SELECT 1 FROM project_participant pp
                    LEFT JOIN review r ON pp.project_id = r.project_id AND pp.user_id = r.user_id
                    WHERE pp.project_id = $1 AND pp.status = 'accepted' AND r.id IS NULL
                )
            )
            "#
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(all_reviewed)
    }
    
    pub async fn update_rating_user(&self, user_id: Uuid) -> Result<(), sqlx::Error>{
        // Meng-update rating global user berdasarkan gabungan dari:
        // Review yang dia dapatkan saat menjadi peserta.
        // Review yang didapatkan oleh project-project buatannya.
        sqlx::query(
            r#"
            UPDATE user_profile 
            SET rating = (
                SELECT COALESCE(AVG(gabungan.rating), 0.0)
                FROM (
                    SELECT rating FROM review WHERE user_id = $1
                    UNION ALL
                    SELECT r.rating FROM review r
                    JOIN projects p ON r.project_id = p.id
                    WHERE p.user_id = $1
                ) AS gabungan
            )
            WHERE user_id = $1
            "#
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_rating_project(&self, project_id: Uuid) -> Result<(), sqlx::Error>{
        sqlx::query(
            r#"
            UPDATE projects 
            SET rating = (
                SELECT COALESCE(AVG(rating), 0.0) 
                FROM review 
                WHERE project_id = $1
            )
            WHERE id = $1
            "#
        )
        .bind(project_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn check_participant_has_reviewed(&self, user_id: Uuid, reviewer_id: Uuid) -> Result<bool, sqlx::Error> {
        let has_reviewed: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 
                FROM review 
                WHERE user_id = $1 AND reviewer_id = $2
            )
            "#
        )
        .bind(user_id)
        .bind(reviewer_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(has_reviewed)
    }
    
    pub async fn check_project_has_reviewed(&self, project_id: Uuid, reviewer_id: Uuid) -> Result<bool, sqlx::Error> {
        let has_reviewed: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 
                FROM review 
                WHERE project_id = $1 AND reviewer_id = $2
            )
            "#
        )
        .bind(project_id)
        .bind(reviewer_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(has_reviewed)
    }
}