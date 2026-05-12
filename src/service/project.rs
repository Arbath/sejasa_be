use axum::extract::{FromRef, FromRequestParts};
use uuid::Uuid;

use crate::{models::{project::{Category, CategoryCreate, CategoryUpdate, Project, ProjectCreate, ProjectUpdate}, user::User}, repository::project::{CategoryRepository, ProjectRepository}, state::AppState, utils::response::AppError};

#[allow(dead_code)]
pub struct ProjectService {
    project_repo: ProjectRepository,
    category_repo: CategoryRepository,
    state: AppState
}

impl ProjectService {
    pub fn new(state: AppState) -> Self {
        let project_repo = ProjectRepository::new(state.database.clone());
        let category_repo = CategoryRepository::new(state.database.clone());

        Self { project_repo, category_repo, state }
    }

    pub async fn get_user_project(&self, user_id: Uuid) -> Result<Vec<Project>, AppError> {
        let query = self.project_repo.find_by_user(user_id).await?;
        Ok(query)
    }
    
    pub async fn get_one_project(&self, project_id: Uuid) -> Result<Project, AppError> {
        let query = self.project_repo.find_by_id(project_id).await?;
        Ok(query)
    }

    pub async fn get_all_project(&self) -> Result<Vec<Project>, AppError> {
        let query = self.project_repo.find_all().await?;
        Ok(query)
    }

    pub async fn add_project(&self, data: ProjectCreate, user: User) -> Result<Project, AppError> {
        let query = self.project_repo.create(data, user.id).await?;
        Ok(query)
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