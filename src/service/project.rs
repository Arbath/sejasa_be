use axum::extract::{FromRef, FromRequestParts};
use uuid::Uuid;

use crate::{models::{project::{Category, CategoryCreate, CategoryUpdate, Project, ProjectCreate, ProjectQueryParams, ProjectUpdate}, user::User}, repository::{project::{CategoryRepository, HastagsRepository, ProjectRepository}, user::UserRepository}, state::AppState, utils::response::AppError};

#[allow(dead_code)]
pub struct ProjectService {
    user_repo: UserRepository,
    project_repo: ProjectRepository,
    category_repo: CategoryRepository,
    hastags_repo: HastagsRepository,
    state: AppState
}

impl ProjectService {
    pub fn new(state: AppState) -> Self {
        let user_repo = UserRepository::new(state.database.clone());
        let project_repo = ProjectRepository::new(state.database.clone());
        let category_repo = CategoryRepository::new(state.database.clone());
        let hastags_repo = HastagsRepository::new(state.database.clone());

        Self { user_repo, project_repo, category_repo, hastags_repo, state }
    }

    pub async fn get_nearest_project(&self, user_id: Uuid, distance: &f64) -> Result<Vec<Project>, AppError> {
        let user = self.user_repo.find_user_profile(&user_id).await?;
        let query = self.project_repo.find_nearest(user.latitude, user.longitude, *distance).await?;
        Ok(query)
    }
    
    pub async fn get_user_project(&self, user_id: Uuid) -> Result<Vec<Project>, AppError> {
        let user_profile = self.user_repo.find_user_profile(&user_id).await?;
        let project = self.project_repo.find_by_user(user_id, user_profile.latitude, user_profile.longitude).await?;
        let res = project;
        Ok(res)
    }
    
    pub async fn get_one_project(&self, project_id: Uuid, user_id: Uuid) -> Result<Project, AppError> {
        let user_profile = self.user_repo.find_user_profile(&user_id).await?;
        let query = self.project_repo.find_by_id(project_id, user_profile.latitude, user_profile.longitude).await?;
        Ok(query)
    }

    pub async fn search_all_project(&self, query: ProjectQueryParams) -> Result<(Vec<Project>, i64), AppError> {
        let query = self.project_repo.search_projects(query).await?;
        Ok(query)
    }
    
    pub async fn get_all_project_unauth(&self) -> Result<Vec<Project>, AppError> {
        let query = self.project_repo.find_all_unauth().await?;
        Ok(query)
    }

    pub async fn get_user_project_unauth(&self, user_id: Uuid) -> Result<Vec<Project>, AppError> {
        let query = self.project_repo.find_by_user_unauth(user_id).await?;
        Ok(query)
    }
    
    pub async fn get_one_project_unauth(&self, project_id: Uuid) -> Result<Project, AppError> {
        let query = self.project_repo.find_by_id_unauth(project_id).await?;
        Ok(query)
    }

    pub async fn add_project(&self, data: ProjectCreate, user: User) -> Result<Project, AppError> {
        let project = self.project_repo.create(data.clone(), user.id).await?;

        let hastags = self.hastags_repo.create_bulk(&data.hastags, &project.id).await?;
        let project_final = project.into_model(hastags);
        Ok(project_final)
    }
    
    pub async fn edit_project(&self, data: ProjectUpdate, project_id: Uuid, user: User) -> Result<Project, AppError> {
        let query = self.project_repo.update(data, project_id, user.id).await?;
        Ok(query)
    }

    pub async fn remove_project(&self, project_id: Uuid, user: User) -> Result<Project, AppError> {
        let query = self.project_repo.delete(project_id, user.id).await?;
        Ok(query)
    }
    
    pub async fn get_all_category(&self) -> Result<Vec<Category>, AppError> {
        let query = self.category_repo.find_all_category().await?;
        Ok(query)
    }
    
    pub async fn get_one_category(&self, name: String) -> Result<Category, AppError> {
        let query = self.category_repo.find_one_category(name).await?;
        Ok(query)
    }

    pub async fn add_category(&self, data: CategoryCreate, user: User) -> Result<Category, AppError> {
        if !user.is_superuser {
            return Err(AppError::AuthError("Only admin can access this resource".to_string()));
        }
        let query = self.category_repo.create(data).await?;
        Ok(query)
    }
    
    pub async fn edit_category(&self, data: CategoryUpdate, category_id: i32, user: User) -> Result<Category, AppError> {
        if !user.is_superuser {
            return Err(AppError::AuthError("Only admin can access this resource".to_string()));
        }
        let query = self.category_repo.update(category_id, data).await?;
        Ok(query)
    }

    pub async fn remove_category(&self, category_id: i32, user: User) -> Result<Category, AppError> {
        if !user.is_superuser {
            return Err(AppError::AuthError("Only admin can access this resource".to_string()));
        }
        let query = self.category_repo.delete(category_id).await?;
        Ok(query)
    }

    pub async fn add_hastags(&self, project_id: Uuid, hastag_name: &String, user: User) -> Result<Vec<String>, AppError> {
        let is_owner = self.project_repo.check_project_ownership(project_id, user.id).await?;
        if !is_owner {
            return Err(AppError::Forbidden("Anda tidak memiliki akses ke project ini".to_string()));
        }

        self.hastags_repo.create(&project_id, hastag_name).await?;
        let list_hastags = self.hastags_repo.find_all_porject_hastags(project_id).await?;
        Ok(list_hastags)
    }
    
    pub async fn remove_hastags(&self, project_id: Uuid, hastag_name: &String, user: User) -> Result<Vec<String>, AppError> {
        let is_owner = self.project_repo.check_project_ownership(project_id, user.id).await?;
        if !is_owner {
            return Err(AppError::Forbidden("Anda tidak memiliki akses ke project ini".to_string()));
        }

        self.hastags_repo.delete(project_id, hastag_name).await?;
        let list_hastags = self.hastags_repo.find_all_porject_hastags(project_id).await?;
        Ok(list_hastags)
    }
}



impl<S> FromRequestParts<S> for ProjectService
where  
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;
    async fn from_request_parts(
        _parts: &mut http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection>
    {
        let state = AppState::from_ref(state);
        Ok(ProjectService::new(state))
    }
}