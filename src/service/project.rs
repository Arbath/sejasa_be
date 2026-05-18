use axum::extract::{FromRef, FromRequestParts};
use uuid::Uuid;

use crate::{models::{project::{Category, CategoryCreate, CategoryUpdate, Project, ProjectCreate, ProjectParticipant, ProjectParticipantCreate, ProjectParticipantStatus, ProjectParticipantUpdate, ProjectQueryParams, ProjectStatus, ProjectUpdate}, review::{Review, ReviewReq}, user::User}, repository::{project::{CategoryRepository, HastagsRepository, ParticipantRepository, ProjectRepository}, review::ReviewRepository, user::UserRepository}, state::AppState, utils::response::AppError};

#[allow(dead_code)]
pub struct ProjectService {
    user_repo: UserRepository,
    project_repo: ProjectRepository,
    category_repo: CategoryRepository,
    hastags_repo: HastagsRepository,
    participant_repo: ParticipantRepository,
    review_repo: ReviewRepository,
    state: AppState
}

impl ProjectService {
    pub fn new(state: AppState) -> Self {
        let user_repo = UserRepository::new(state.database.clone());
        let project_repo = ProjectRepository::new(state.database.clone());
        let category_repo = CategoryRepository::new(state.database.clone());
        let hastags_repo = HastagsRepository::new(state.database.clone());
        let participant_repo = ParticipantRepository::new(state.database.clone());
        let review_repo = ReviewRepository::new(state.database.clone());

        Self { user_repo, project_repo, category_repo, hastags_repo, participant_repo, review_repo, state }
    }
    
    pub async fn get_user_project(&self, user_id: Uuid) -> Result<Vec<Project>, AppError> {
        let user_profile = self.user_repo.find_user_profile(&user_id).await?;
        let project = self.project_repo.find_by_user(user_id, user_profile.latitude, user_profile.longitude).await?;
        let res = project;
        Ok(res)
    }
    
    pub async fn get_one_project(&self, project_id: Uuid, user_id: Uuid) -> Result<Project, AppError> {
        let user_profile = self.user_repo.find_user_profile(&user_id).await?;
        let project = self.project_repo.find_by_id(project_id, Some(user_profile.latitude), Some(user_profile.longitude)).await?;
        Ok(project)
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
        match &data.status{
            Some(ProjectStatus::DONE) => {
                let reviewed = self.review_repo.check_all_accepted_participants_reviewed(project_id).await?;
                if !reviewed {
                    return Err(AppError::Forbidden("Anda harus memberikan review kepada participant terlebih dahulu!".to_string()));
                }
            },
            Some(ProjectStatus::CANCEL) => {
                self.participant_repo.update_status_all(project_id, ProjectParticipantStatus::REJECTED).await?;
            },
            Some(ProjectStatus::PENDING) => {
                // self.participant_repo.update_status_all(project_id, ProjectParticipantStatus::PENDING).await?;
            },
            _ => {}
        };
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

    pub async fn apply_project(&self, project_id: Uuid, user: User) -> Result<Project, AppError> {
        let is_owner = self.project_repo.check_project_ownership(project_id, user.id).await?;
        if is_owner {
            return Err(AppError::Forbidden("Anda tidak bisa apply project milik Anda!".to_string()));
        }
        let project_status = self.project_repo.check_status(project_id).await?;
        if project_status != ProjectStatus::HIRING {
            return Err(AppError::Forbidden("Proyek tidak sedang mencari partisipan!".to_string()));
        }
        let status = ProjectParticipantStatus::PENDING;
        let data = ProjectParticipantCreate {
            user_id: user.id,
            project_id: project_id,
            status: status
        };
        let _ = self.participant_repo.create(data).await?;
        let user_profile = self.user_repo.find_user_profile(&user.id).await?;
        let project =  self.project_repo.find_by_id(project_id, Some(user_profile.latitude), Some(user_profile.longitude)).await?;
        Ok(project)
    }

    pub async fn list_participant_project(&self, project_id: Uuid) -> Result<Vec<ProjectParticipant>, AppError> {
        let q = self.participant_repo.find_all(project_id).await?;
        Ok(q)
    }
    
    pub async fn apply_participant_project(&self, project_id: Uuid, project_part_id: i32, user: User, data: ProjectParticipantUpdate) -> Result<ProjectParticipant, AppError> {
        let is_owner = self.project_repo.check_project_ownership(project_id, user.id).await?;
        if !is_owner {
            return Err(AppError::Forbidden("Anda tidak memiliki akses ke project ini".to_string()));
        }
        let q = self.participant_repo.update_status(data, project_part_id).await?;
        Ok(q)
    }

    pub async fn review_participant(&self, project_id: Uuid, participant_id: Uuid, data: ReviewReq, user: User) -> Result<Review, AppError> {
        let is_owner = self.project_repo.check_project_ownership(project_id, user.id).await?;
        if !is_owner {
            return Err(AppError::Forbidden("Anda tidak memiliki akses ke project ini".to_string()));
        }
        let has_reviewed = self.review_repo.check_participant_has_reviewed(participant_id, user.id).await?;
        if has_reviewed {
            return Err(AppError::BadRequest("Anda sudah memberikan review untuk participant ini!".to_string()));
        }
        if data.rating > 5.0 || data.rating < 0.0 {
            return Err(AppError::BadRequest("Rating berkisar antara 0.0 - 5.0".to_string()));
        };

        let q = self.review_repo.create(participant_id, project_id, user.id, data).await?;
        let _ = self.review_repo.update_rating_user(q.user_id).await?;

        Ok(q)
    }

    pub async fn review_all_participant(&self, project_id: Uuid, data: ReviewReq, user: User) -> Result<Vec<Review>, AppError> {
        let is_owner = self.project_repo.check_project_ownership(project_id, user.id).await?;
        if !is_owner {
            return Err(AppError::Forbidden("Anda tidak memiliki akses ke project ini".to_string()));
        }
        if data.rating > 5.0 || data.rating < 0.0 {
            return Err(AppError::BadRequest("Rating berkisar antara 0.0 - 5.0".to_string()));
        };

        let q = self.review_repo.create_all_participant( project_id, user.id, data).await?;

        Ok(q)
    }

    pub async fn review_project(&self, project_id: Uuid, data: ReviewReq, user: User) -> Result<Review, AppError> {
        let is_participant = self.participant_repo.check_participant_status(project_id, user.id).await?;
        if !is_participant {
            return Err(AppError::Forbidden("Anda bukan participant project ini".to_string()));
        }
        let has_reviewed = self.review_repo.check_project_has_reviewed(project_id, user.id).await?;
        if has_reviewed {
            return Err(AppError::BadRequest("Anda sudah memberikan review untuk project ini!".to_string()));
        }
        if data.rating > 5.0 || data.rating < 0.0 {
            return Err(AppError::BadRequest("Rating berkisar antara 0.0 - 5.0".to_string()));
        };
        
        let project = self.project_repo.find_by_id(project_id, None, None).await?;
        let q = self.review_repo.create(project.user_id , project_id, user.id, data).await?;
        let _ = self.review_repo.update_rating_project(q.project_id).await?;
        let _ = self.review_repo.update_rating_user(project.user_id).await?;
        Ok(q)
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