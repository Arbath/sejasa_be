use sqlx::PgPool;
use uuid::Uuid;
use crate::models::user::{UpdateProfileReq, User, UserCreate, UserProfile, UserSkill, UserSkillCreate, UserSkillUpdate};
use chrono::{DateTime, Utc};

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: &Uuid) -> Result<User, sqlx::Error> {
        
        sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_by_name(
        &self,
        name: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE name = $1"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_email(
        &self,
        email: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_name_or_email(
        &self,
        identifier: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE name = $1 OR email = $1"
        )
        .bind(identifier)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn update_profile(&self, user: User) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET name = $1,
                email = $2,
                is_superuser = $3
            WHERE id = $4
            RETURNING *
            "#
        )
        .bind(user.name)
        .bind(user.email)
        .bind(user.is_superuser)
        .bind(user.id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_all(&self) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            ORDER BY id ASC
            "#
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn create(&self, data: UserCreate) -> Result<User, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let new_user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (name, password, email, account_type)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#
        )
        .bind(data.name)
        .bind(data.password)
        .bind(data.email)
        .bind(data.account_type)
        .fetch_one(&mut *tx) 
        .await?;

        sqlx::query(
            r#"
            INSERT INTO user_profile (user_id, gender, latitude, longitude)
            VALUES ($1, $2, $3, $4)
            "#
        )
        .bind(new_user.id)
        .bind(data.gender)
        .bind(data.latitude)
        .bind(data.longitude)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(new_user)
    }

    pub async fn update(&self, user_id: Uuid, data: UpdateProfileReq) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
            UPDATE users 
            SET name = COALESCE($1, name),
                email = COALESCE($2, email)
            WHERE id = $3
            "#
        )
        .bind(data.name)
        .bind(data.email)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO user_profile (user_id, gender, contact, address, latitude, longitude, image)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (user_id) DO UPDATE 
            SET gender = COALESCE(EXCLUDED.gender, user_profile.gender),
                contact = COALESCE(EXCLUDED.contact, user_profile.contact),
                address = COALESCE(EXCLUDED.address, user_profile.address),
                latitude = COALESCE(EXCLUDED.latitude, user_profile.latitude),
                longitude = COALESCE(EXCLUDED.longitude, user_profile.longitude),
                image = COALESCE(EXCLUDED.image, user_profile.image)
            "#
        )
        .bind(user_id)
        .bind(data.gender)
        .bind(data.contact)
        .bind(data.address)
        .bind(data.latitude)
        .bind(data.longitude)
        .bind(data.image)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn delete(&self, user_id: &i32) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            DELETE FROM users
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_user_profile(&self, user_id: &Uuid) -> Result<UserProfile, sqlx::Error> {
        sqlx::query_as::<_, UserProfile>(
        r#"
            SELECT 
                u.id, u.name, u.email, u.created_at, u.account_type,
                p.gender, p.rating, p.contact, p.latitude, p.longitude, p.image, p.address
            FROM users u
            LEFT JOIN user_profile p ON u.id = p.user_id
            WHERE u.id = $1
            "#
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
    }
    
    pub async fn search_user(&self, name: &String) -> Result<Vec<User>, sqlx::Error> {
        let user_name = format!("%{}%", name);
        sqlx::query_as::<_, User>(
        r#"
            SELECT * FROM users WHERE name ILIKE $1 ORDER BY name ASC
            "#
        )
        .bind(user_name)
        .fetch_all(&self.pool)
        .await
    }
    
    pub async fn find_all_users(&self) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
        r#"
            SELECT * FROM users ORDER BY name ASC
            "#
        )
        .fetch_all(&self.pool)
        .await
    }
}

pub struct TokenRepository {
    pool: PgPool,
}

impl TokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save_token(
        &self,
        token: &str,
        user_id: Uuid,
        expires_at: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (token, user_id, expires_at)
            VALUES ($1, $2, $3)
            "#
        )
        .bind(token)
        .bind(user_id)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn exists(&self, token: &str) -> Result<bool, sqlx::Error> {
        let exists: Option<i32> = sqlx::query_scalar(
            r#"
            SELECT 1
            FROM refresh_tokens
            WHERE token = $1
            LIMIT 1
            "#
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;

        Ok(exists.is_some())
    }

    pub async fn revoke(&self, token: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "DELETE FROM refresh_tokens WHERE token = $1"
        )
        .bind(token)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

pub struct UserSkillsRepository {
    pool: PgPool
}

impl UserSkillsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_skills_by_user(&self, user_id: &Uuid) -> Result<Vec<UserSkill>, sqlx::Error> {
        sqlx::query_as::<_, UserSkill>(
            r#"SELECT * FROM user_skills WHERE user_id = $1"#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }
    
    pub async fn find_all_skills(&self) -> Result<Vec<UserSkill>, sqlx::Error> {
        sqlx::query_as::<_, UserSkill>(
            r#"SELECT * FROM user_skills LIMIT 10"#
        )
        .fetch_all(&self.pool)
        .await
    }
    
    pub async fn find_skills_by_id(&self, skill_id: i32) -> Result<UserSkill, sqlx::Error> {
        sqlx::query_as::<_, UserSkill>(
            r#"SELECT * FROM user_skills WHERE id = $1"#
        )
        .bind(skill_id)
        .fetch_one(&self.pool)
        .await
    }
    
    pub async fn find_skills_by_name(&self, skill_name: String) -> Result<Vec<UserSkill>, sqlx::Error> {
        let search_term = format!("%{}%", skill_name);
        sqlx::query_as::<_, UserSkill>(
            r#"SELECT * FROM user_skills WHERE name ILIKE $1"#
        )
        .bind(search_term)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn create_skills(&self, user_id: Uuid, data: UserSkillCreate) -> Result<UserSkill, sqlx::Error> {
        sqlx::query_as::<_, UserSkill>(
            r#"INSERT INTO user_skills (name, descriptions, user_id) VALUES($1, $2, $3) RETURNING * "#
        )
        .bind(data.name)
        .bind(data.descriptions)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update_skills(&self, user_id: Uuid, skill_id: i32, data: UserSkillUpdate) -> Result<UserSkill, sqlx::Error> {
        sqlx::query_as::<_,UserSkill>(
            r#"UPDATE user_skills 
                SET 
                    name = COALESCE($1, name),
                    descriptions = COALESCE($2, descriptions)
                WHERE id = $3 AND user_id = $4 RETURNING *"#
        )
        .bind(data.name)
        .bind(data.descriptions)
        .bind(skill_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
    }
    
    pub async fn delete_skills(&self, user_id: Uuid, skill_id: i32) -> Result<UserSkill, sqlx::Error> {
        sqlx::query_as::<_,UserSkill>(
            r#"DELETE FROM user_skills WHERE id = $1 AND user_id = $2 RETURNING *"#
        )
        .bind(skill_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
    }
}