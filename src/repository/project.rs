use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::models::project::*;

pub struct ProjectRepository {
    pool: PgPool
}

impl ProjectRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn search_projects(
        &self,
        params: ProjectQueryParams,
    ) -> Result<(Vec<Project>, i64), sqlx::Error> { // Return tuple (Data, Total Items)
        let page = params.page.unwrap_or(1);
        let limit = params.limit.unwrap_or(10);
        let offset = (page - 1) * limit;

        // Inisialisasi Base Query
        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            WITH ParticipantCount AS (
                SELECT project_id, COUNT(id) AS count_current_participant
                FROM project_participant
                GROUP BY project_id
            ),
            HashtagList AS (
                SELECT project_id, ARRAY_AGG(name) AS hastag_names
                FROM hastags
                GROUP BY project_id
            )
            SELECT 
                p.*, 
                COALESCE(pc.count_current_participant, 0) AS cur_participant,
                COALESCE(hl.hastag_names, '{}') AS hastags,
                c.name AS category_name,
                u.name AS owner_name,
                COALESCE(up.rating, 0.0) AS owner_rating,
                up.image AS owner_image,
                
                COUNT(*) OVER() AS total_items
            "#
        );

        let has_location = params.lat.is_some() && params.lon.is_some();

        // kolom virtual `distance_meters` HANYA jika ada lat/lon
        if let (Some(lat), Some(lon)) = (params.lat, params.lon) {
            builder.push(", (6371000 * acos(cos(radians(");
            builder.push_bind(lat);
            builder.push(")) * cos(radians(p.latitude)) * cos(radians(p.longitude) - radians(");
            builder.push_bind(lon);
            builder.push(")) + sin(radians(");
            builder.push_bind(lat);
            builder.push(")) * sin(radians(p.latitude)))) AS distance_meters");
        }

        builder.push(
            r#"
            FROM projects p
            LEFT JOIN category c ON p.category_id = c.id
            LEFT JOIN users u ON p.user_id = u.id
            LEFT JOIN user_profile up ON p.user_id = up.user_id
            LEFT JOIN ParticipantCount pc ON p.id = pc.project_id
            LEFT JOIN HashtagList hl ON p.id = hl.project_id
            WHERE 1=1
            "#
        );

        if let Some(q) = params.q.clone() {
            builder.push(" AND p.name ILIKE ");
            builder.push_bind(format!("%{}%", q));
        }

        if let Some(status) = params.status.clone() {
            builder.push(" AND p.status = ");
            builder.push_bind(status); 
        }

        if let Some(cat_str) = params.category.clone() {
            let categories: Vec<String> = cat_str.split(',')
                .map(|s| s.trim().to_string())
                .collect();
            
            builder.push(" AND c.name ILIKE ANY(");
            builder.push_bind(categories);
            builder.push(")");
        }

        if let (Some(lat), Some(lon), Some(dist)) = (params.lat, params.lon, params.distance) {
            builder.push(" AND p.latitude IS NOT NULL AND p.longitude IS NOT NULL");
            builder.push(" AND (6371000 * acos(cos(radians(");
            builder.push_bind(lat);
            builder.push(")) * cos(radians(p.latitude)) * cos(radians(p.longitude) - radians(");
            builder.push_bind(lon);
            builder.push(")) + sin(radians(");
            builder.push_bind(lat);
            builder.push(")) * sin(radians(p.latitude)))) <= ");
            builder.push_bind(dist);
        }

        // Sorting
        match params.sort.as_deref() {
            Some("oldest") => {
                builder.push(" ORDER BY p.created_at ASC");
            }
            Some("closest") if has_location => {
                builder.push(" ORDER BY distance_meters ASC");
            }
            _ => {
                builder.push(" ORDER BY p.created_at DESC");
            }
        }

        // pagination
        builder.push(" LIMIT ");
        builder.push_bind(limit);
        builder.push(" OFFSET ");
        builder.push_bind(offset);

        let query = builder.build_query_as::<ProjectPaginationRow>();
        let rows = query.fetch_all(&self.pool).await?;

        let total_items = rows.first().map(|r| r.total_items).unwrap_or(0);
        
        // Vec<ProjectPaginationRow> to Vec<Project>
        let projects: Vec<Project> = rows.into_iter().map(|r| r.project).collect();

        Ok((projects, total_items))
    }

    pub async fn find_nearest(&self, latitude: f64, longitude: f64, distance: f64) -> Result<Vec<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            r#"
            WITH ParticipantCount AS (
                SELECT project_id, COUNT(id) AS count_current_participant
                FROM project_participant
                GROUP BY project_id
            ),
            HashtagList AS (
                SELECT project_id, ARRAY_AGG(name) AS hastag_names
                FROM hastags
                GROUP BY project_id
            )
            SELECT 
                p.*, 
                -- Hitung jarak dan jadikan kolom virtual bernama 'distance_meters'
                (
                    6371000 * acos(
                        cos(radians($1)) * cos(radians(p.latitude)) * cos(radians(p.longitude) - radians($2)) + 
                        sin(radians($1)) * sin(radians(p.latitude))
                    )
                ) AS distance_meters,
                 COALESCE(pc.count_current_participant, 0) AS cur_participant,
                COALESCE(hl.hastag_names, '{}') AS hastags,
                 c.name AS category_name,
                 u.name AS owner_name,
                 up.rating AS owner_rating,
                 up.image AS owner_image
            FROM projects p
            LEFT JOIN category c ON p.category_id = c.id
            LEFT JOIN users u ON p.user_id = u.id
            LEFT JOIN user_profile up ON p.user_id = up.user_id
            LEFT JOIN ParticipantCount pc ON p.id = pc.project_id
            LEFT JOIN HashtagList hl ON p.id = hl.project_id
            WHERE p.latitude IS NOT NULL AND p.longitude IS NOT NULL
            -- Filter agar hanya mengambil yang di dalam radius
            AND (
                6371000 * acos(
                    cos(radians($1)) * cos(radians(p.latitude)) * cos(radians(p.longitude) - radians($2)) + 
                    sin(radians($1)) * sin(radians(p.latitude))
                )
            ) <= $3
            -- Urutkan dari yang lokasinya paling dekat dengan user
            ORDER BY distance_meters ASC
            "#
        )
        .bind(latitude)
        .bind(longitude)
        .bind(distance)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_all(&self, latitude: f64, longitude: f64) -> Result<Vec<Project>, sqlx::Error>{
        sqlx::query_as::<_, Project>(
            r#"
            WITH ParticipantCount AS (
                SELECT project_id, COUNT(id) AS count_current_participant
                FROM project_participant
                GROUP BY project_id
            ),
            HashtagList AS (
                SELECT project_id, ARRAY_AGG(name) AS hastag_names
                FROM hastags
                GROUP BY project_id
            )
            SELECT 
            p.* ,(
                    6371000 * acos(
                        cos(radians($1)) * cos(radians(p.latitude)) * cos(radians(p.longitude) - radians($2)) + 
                        sin(radians($1)) * sin(radians(p.latitude))
                    )
                ) AS distance_meters,
            COALESCE(pc.count_current_participant, 0) AS cur_participant,
            COALESCE(hl.hastag_names, '{}') AS hastags,
            c.name AS category_name,
            u.name AS owner_name,
            up.rating AS owner_rating,
            up.image AS owner_image
            FROM projects p
            LEFT JOIN category c ON p.category_id = c.id
            LEFT JOIN users u ON p.user_id = u.id
            LEFT JOIN user_profile up ON p.user_id = up.user_id
            LEFT JOIN ParticipantCount pc ON p.id = pc.project_id
            LEFT JOIN HashtagList hl ON p.id = hl.project_id
            ORDER BY created_at DESC"#
        )
        .bind(latitude)
        .bind(longitude)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, project_id: Uuid, latitude: f64, longitude: f64) -> Result<Project, sqlx::Error>{
        sqlx::query_as::<_, Project>(
            r#"
            WITH ParticipantCount AS (
                SELECT project_id, COUNT(id) AS count_current_participant
                FROM project_participant
                GROUP BY project_id
            ),
            HashtagList AS (
                SELECT project_id, ARRAY_AGG(name) AS hastag_names
                FROM hastags
                GROUP BY project_id
            )
            SELECT 
                p.* ,(
                    6371000 * acos(
                        cos(radians($1)) * cos(radians(p.latitude)) * cos(radians(p.longitude) - radians($2)) + 
                        sin(radians($1)) * sin(radians(p.latitude))
                    )
                ) AS distance_meters,
                COALESCE(pc.count_current_participant, 0) AS cur_participant,
                COALESCE(hl.hastag_names, '{}') AS hastags,
                c.name AS category_name,
                u.name AS owner_name,
                up.rating AS owner_rating,
                up.image AS owner_image
                FROM projects p
                LEFT JOIN category c ON p.category_id = c.id
                LEFT JOIN users u ON p.user_id = u.id
                LEFT JOIN user_profile up ON p.user_id = up.user_id
                LEFT JOIN ParticipantCount pc ON p.id = pc.project_id
                LEFT JOIN HashtagList hl ON p.id = hl.project_id
                WHERE p.id = $3
            "#
        )
        .bind(latitude)
        .bind(longitude)
        .bind(project_id)
        .fetch_one(&self.pool)
        .await
    }
    
    pub async fn find_by_id_unauth(&self, project_id: Uuid) -> Result<Project, sqlx::Error>{
        sqlx::query_as::<_, _>(
            r#"
            WITH ParticipantCount AS (
                SELECT project_id, COUNT(id) AS count_current_participant
                FROM project_participant
                GROUP BY project_id
            ),
            HashtagList AS (
                SELECT project_id, ARRAY_AGG(name) AS hastag_names
                FROM hastags
                GROUP BY project_id
            )
            SELECT p.* , 
                COALESCE(pc.count_current_participant, 0) AS cur_participant,
                COALESCE(hl.hastag_names, '{}') AS hastags,
                c.name AS category_name,
                u.name AS owner_name,
                up.rating AS owner_rating,
                up.image AS owner_image
                FROM projects p
                LEFT JOIN category c ON p.category_id = c.id
                LEFT JOIN users u ON p.user_id = u.id
                LEFT JOIN user_profile up ON p.user_id = up.user_id
                LEFT JOIN ParticipantCount pc ON p.id = pc.project_id
                LEFT JOIN HashtagList hl ON p.id = hl.project_id
                WHERE p.id = $1
            ORDER BY created_at DESC"#
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await
    }
    pub async fn find_by_user_unauth(&self, user_id: Uuid) -> Result<Vec<Project>, sqlx::Error>{
        sqlx::query_as::<_, _>(
            r#"
            WITH ParticipantCount AS (
                SELECT project_id, COUNT(id) AS count_current_participant
                FROM project_participant
                GROUP BY project_id
            ),
            HashtagList AS (
                SELECT project_id, ARRAY_AGG(name) AS hastag_names
                FROM hastags
                GROUP BY project_id
            )
            SELECT p.* ,
                COALESCE(pc.count_current_participant, 0) AS cur_participant,
                COALESCE(hl.hastag_names, '{}') AS hastags,
                c.name AS category_name,
                u.name AS owner_name,
                up.rating AS owner_rating,
                up.image AS owner_image
                FROM projects p
                LEFT JOIN category c ON p.category_id = c.id
                LEFT JOIN users u ON p.user_id = u.id
                LEFT JOIN user_profile up ON p.user_id = up.user_id
                LEFT JOIN ParticipantCount pc ON p.id = pc.project_id
                LEFT JOIN HashtagList hl ON p.id = hl.project_id
                WHERE p.user_id = $1
            ORDER BY created_at DESC"#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }
    pub async fn find_all_unauth(&self) -> Result<Vec<Project>, sqlx::Error>{
        sqlx::query_as::<_, _>(
            r#"
            WITH ParticipantCount AS (
                SELECT project_id, COUNT(id) AS count_current_participant
                FROM project_participant
                GROUP BY project_id
            ),
            HashtagList AS (
                SELECT project_id, ARRAY_AGG(name) AS hastag_names
                FROM hastags
                GROUP BY project_id
            )
            SELECT
                p.* ,
                COALESCE(pc.count_current_participant, 0) AS cur_participant,
                COALESCE(hl.hastag_names, '{}') AS hastags,
                c.name AS category_name,
                u.name AS owner_name,
                up.rating AS owner_rating,
                up.image AS owner_image
                FROM projects p
                LEFT JOIN category c ON p.category_id = c.id
                LEFT JOIN users u ON p.user_id = u.id
                LEFT JOIN user_profile up ON p.user_id = up.user_id
                LEFT JOIN ParticipantCount pc ON p.id = pc.project_id
                LEFT JOIN HashtagList hl ON p.id = hl.project_id
            ORDER BY created_at DESC"#
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn check_project_ownership(&self, project_id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
        
        let has_permission: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM projects 
                WHERE id = $1 AND user_id = $2
            )
            "#
        )
        .bind(project_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(has_permission)
    }
    
    pub async fn find_by_user(&self, user_id: Uuid, latitude: f64, longitude: f64) -> Result<Vec<Project>, sqlx::Error>{
        sqlx::query_as::<_, Project>(
            r#"
            WITH ParticipantCount AS (
                SELECT project_id, COUNT(id) AS count_current_participant
                FROM project_participant
                GROUP BY project_id
            ),
            HashtagList AS (
                SELECT project_id, ARRAY_AGG(name) AS hastag_names
                FROM hastags
                GROUP BY project_id
            )
            SELECT 
                p.* ,(
                    6371000 * acos(
                        cos(radians($1)) * cos(radians(p.latitude)) * cos(radians(p.longitude) - radians($2)) + 
                        sin(radians($1)) * sin(radians(p.latitude))
                    )
                ) AS distance_meters,
                COALESCE(pc.count_current_participant, 0) AS cur_participant,
                COALESCE(hl.hastag_names, '{}') AS hastags,
                c.name AS category_name,
                u.name AS owner_name,
                up.rating AS owner_rating,
                up.image AS owner_image
                FROM projects p
                LEFT JOIN category c ON p.category_id = c.id
                LEFT JOIN users u ON p.user_id = u.id
                LEFT JOIN user_profile up ON p.user_id = up.user_id
                LEFT JOIN ParticipantCount pc ON p.id = pc.project_id
                LEFT JOIN HashtagList hl ON p.id = hl.project_id
                WHERE p.user_id = $3
            "#
        )
        .bind(latitude)
        .bind(longitude)
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

pub struct HastagsRepository {
    pool: PgPool
}

impl HastagsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_all_hastags(&self) -> Result<Vec<Hastags>, sqlx::Error> {
        sqlx::query_as::<_, Hastags>(
            r#"SELECT * FROM hastags"#
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_all_porject_hastags(&self, project_id: Uuid) -> Result<Vec<String>, sqlx::Error> {
        sqlx::query_scalar::<_, String>(
            r#"SELECT name FROM hastags WHERE project_id = $1"#
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
    }
    
    pub async fn find_one_hastags(&self, name: String) -> Result<Hastags, sqlx::Error> {
        let search_name = format!("%{}%", name);
        sqlx::query_as::<_, Hastags>(
            r#"SELECT * FROM hastags WHERE name ILIKE $1"#
        )
        .bind(search_name)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn create(&self, project_id: &Uuid, name: &String) -> Result<Hastags, sqlx::Error>{
        sqlx::query_as::<_, Hastags>(
            r#"INSERT INTO hastags(name, project_id) VALUES($1, $2) RETURNING *"#
        )
        .bind(name)
        .bind(project_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn create_bulk(&self, data: &Option<Vec<String>>, project_id: &Uuid) -> Result<Vec<String>, sqlx::Error> {
        let tags = match data {
            Some(t) if !t.is_empty() => t,
            _ => return Ok(vec![]),
        };

        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO hastags (name, project_id) "
        );

        query_builder.push_values(tags, |mut b, name| {
            b.push_bind(name)
            .push_bind(project_id);
        });

        query_builder.push(" RETURNING name");

        let query = query_builder.build_query_scalar::<String>();
        let inserted_names = query.fetch_all(&self.pool).await?;

        Ok(inserted_names)
    }
    
    pub async fn delete(&self, project_id: Uuid, name: &String) -> Result<Hastags, sqlx::Error>{
        sqlx::query_as::<_, Hastags>(
            r#"DELETE FROM hastags WHERE project_id = $1 AND name = $2 RETURNING *"#
        )
        .bind(project_id)
        .bind(name)
        .fetch_one(&self.pool)
        .await
    }
}