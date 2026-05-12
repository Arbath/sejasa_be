use sqlx::PgPool;
use uuid::Uuid;

use crate::models::project::{Category, CategoryCreate, CategoryUpdate, Project, ProjectCreate, ProjectUpdate};

pub struct ProjectRepository {
    pool: PgPool
}

impl ProjectRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_all(&self) -> Result<Vec<Project>, sqlx::Error>{
        sqlx::query_as::<_, Project>(
            r#"SELECT * FROM projects ORDER BY created_at DESC"#
        )
        .fetch_all(&self.pool)
        .await
    }
    pub async fn find_by_id(&self, project_id: Uuid) -> Result<Project, sqlx::Error>{
        sqlx::query_as::<_, Project>(
            r#"SELECT * FROM projects WHERE id = $1"#
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await
    }
    
    pub async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Project>, sqlx::Error>{
        sqlx::query_as::<_, Project>(
            r#"SELECT * FROM projects WHERE user_id = $1"#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn is_slug_used(&self, slug: &str, exclude_id: Option<Uuid>) -> Result<bool, sqlx::Error> {
        let exists: (bool,) = match exclude_id {
            Some(id) => {
                // Update: Cek apakah slug dipakai oleh project LAIN
                sqlx::query_as("SELECT EXISTS(SELECT 1 FROM projects WHERE slug = $1 AND id != $2)")
                    .bind(slug)
                    .bind(id)
                    .fetch_one(&self.pool)
                    .await?
            },
            None => {
                // Create: Cek ke seluruh tabel
                sqlx::query_as("SELECT EXISTS(SELECT 1 FROM projects WHERE slug = $1)")
                    .bind(slug)
                    .fetch_one(&self.pool)
                    .await?
            }
        };
        
        Ok(exists.0)
    }

    pub async fn create(&self,data: ProjectCreate, user_id: Uuid) -> Result<Project, sqlx::Error>{
        sqlx::query_as::<_, Project>(
            r#"INSERT INTO projects (name, address, max_participant, status, descriptions, requirements, slug, latitude, longitude, category_id, user_id)
                VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) 
                RETURNING *
            "#
        )
        .bind(data.name)
        .bind(data.address)
        .bind(data.max_participant)
        .bind(data.status)
        .bind(data.descriptions)
        .bind(data.requirements)
        .bind(data.slug)
        .bind(data.latitude)
        .bind(data.longitude)
        .bind(data.category_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
    }
    pub async fn update(&self, data: ProjectUpdate, project_id: Uuid, user_id: Uuid) -> Result<Project, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            r#"
            UPDATE projects 
            SET 
                name = COALESCE($1, name),
                address = COALESCE($2, address),
                max_participant = COALESCE($3, max_participant),
                status = COALESCE($4, status),
                descriptions = COALESCE($5, descriptions),
                requirements = COALESCE($6, requirements),
                slug = COALESCE($7, slug),
                latitude = COALESCE($8, latitude),
                longitude = COALESCE($9, longitude),
                category_id = COALESCE($10, category_id),
                updated_at = now()
            WHERE id = $11 AND user_id = $12
            RETURNING *
            "#
        )
        .bind(data.name)
        .bind(data.address)
        .bind(data.max_participant)
        .bind(data.status)
        .bind(data.descriptions)
        .bind(data.requirements)
        .bind(data.slug)
        .bind(data.latitude)
        .bind(data.longitude)
        .bind(data.category_id)
        .bind(project_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn delete(&self, project_id: Uuid, user_id: Uuid) -> Result<Project, sqlx::Error>{
        sqlx::query_as::<_, Project>(
            r#"DELETE FROM projects WHERE id = $1 AND user_id = $2 RETURNING *"#
        )
        .bind(project_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
    }
}

pub struct CategoryRepository {
    pool: PgPool
}

impl CategoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_all_category(&self) -> Result<Vec<Category>, sqlx::Error> {
        sqlx::query_as::<_, Category>(
            r#"SELECT * FROM category"#
        )
        .fetch_all(&self.pool)
        .await
    }
    
    pub async fn find_one_category(&self, name: String) -> Result<Category, sqlx::Error> {
        let search_name = format!("%{}%", name);
        sqlx::query_as::<_, Category>(
            r#"SELECT * FROM category WHERE name ILIKE $1"#
        )
        .bind(search_name)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn create(&self, data: CategoryCreate) -> Result<Category, sqlx::Error>{
        sqlx::query_as::<_, Category>(
            r#"INSERT INTO category(name, descriptions) VALUES($1, $2) RETURNING *"#
        )
        .bind(data.name)
        .bind(data.descriptions)
        .fetch_one(&self.pool)
        .await
    }
    
    pub async fn update(&self, category_id: i32, data: CategoryUpdate) -> Result<Category, sqlx::Error>{
        sqlx::query_as::<_, Category>(
            r#"UPDATE category
            SET
                name = COALESCE($1, name),
                descriptions = COALESCE($2, descriptions) 
            WHERE id = $3 RETURNING *"#
        )
        .bind(data.name)
        .bind(data.descriptions)
        .bind(category_id)
        .fetch_one(&self.pool)
        .await
    }
    
    pub async fn delete(&self, category_id: i32) -> Result<Category, sqlx::Error>{
        sqlx::query_as::<_, Category>(
            r#"DELETE FROM category WHERE id = $1 RETURNING *"#
        )
        .bind(category_id)
        .fetch_one(&self.pool)
        .await
    }
}