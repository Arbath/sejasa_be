use axum::extract::{FromRef, FromRequestParts};
use uuid::Uuid;

use crate::{models::user::{UpdateProfileReq, User, UserCreate, UserProfileRes, UserReq, UserSkill, UserSkillCreate, UserSkillUpdate}, repository::user::{UserRepository, UserSkillsRepository}, state::AppState, utils::response::AppError};

#[allow(dead_code)]
pub struct UserService {
    user_repo: UserRepository,
    skill_repo: UserSkillsRepository,
    state: AppState
}

impl UserService {
    pub fn new(state: AppState) -> Self {
        let user_repo = UserRepository::new(state.database.clone());
        let skill_repo = UserSkillsRepository::new(state.database.clone());

        Self {user_repo, skill_repo, state}
    }

    pub async fn user_profile(&self, user_id: Uuid) -> Result<UserProfileRes, AppError> {
        let user = self.user_repo.find_user_profile(&user_id)
            .await.map_err(|e| AppError::NotFound(format!("User not found! {e}")))?;

        let skills = self.skill_repo.find_skills_by_user(&user_id).await?;
        let user_profile = UserProfileRes::combine(user, skills);
        Ok(user_profile)
    }

    pub async fn user_register(&self, data: UserReq) -> Result<User, AppError> {
        if data.password1 != data.password2 {
            return Err(AppError::BadRequest("Password tidak sama!".to_string()));
        }

        let password_to_hash = data.password1.clone();
        let password_hash = tokio::task::spawn_blocking(move || {
            crate::utils::hash::generate(&password_to_hash) 
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Thread error: {}", e)))??;

        let user_to_save = UserCreate::from_req(data, password_hash);
        let created_user = self.user_repo.create(user_to_save).await.map_err(|e| AppError::BadRequest(format!("{e}")))?;

        Ok(created_user)
    }

    pub async fn user_update(&self, user: User, data: UpdateProfileReq) -> Result<UserProfileRes, AppError>{
        let _ = self.user_repo.update(user.id, data)
            .await.map_err(|e| AppError::BadRequest(format!{"Gagal update profile: {e}"}))?;

        let user_profile = self.user_repo.find_user_profile(&user.id).await?;
        let skills = self.skill_repo.find_skills_by_user(&user.id).await?;
        let user_profile_final = UserProfileRes::combine(user_profile, skills);
        Ok(user_profile_final)
    }

    pub async fn find_skills_name(&self, name: &String) -> Result<Vec<UserSkill>, AppError> {
        let skils = self.skill_repo.find_skills_by_name(name.clone()).await?;

        Ok(skils)
    }
    
    pub async fn find_all_skills(&self) -> Result<Vec<UserSkill>, AppError> {
        let skils = self.skill_repo.find_all_skills().await?;

        Ok(skils)
    }

    pub async fn add_my_skills(&self, user: User, data: UserSkillCreate) -> Result<UserSkill, AppError> {
        let skils = self.skill_repo.create_skills(user.id, data).await?;
        
        Ok(skils)
    }
    
    pub async fn edit_my_skills(&self, user: User, skill_id: i32, data: UserSkillUpdate) -> Result<UserSkill, AppError> {
        let skils = self.skill_repo.update_skills(user.id, skill_id, data).await?;
        
        Ok(skils)
    }
    
    pub async fn remove_my_skills(&self, user: User, skill_id: i32) -> Result<UserSkill, AppError> {
        let skils = self.skill_repo.delete_skills(user.id, skill_id).await?;
        
        Ok(skils)
    }
}

impl<S> FromRequestParts<S> for UserService
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
        Ok(UserService::new(state))
    }
}
