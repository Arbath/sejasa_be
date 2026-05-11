use axum::extract::{FromRef, FromRequestParts};
use uuid::Uuid;

use crate::{models::user::{UpdateProfileReq, User, UserCreate, UserProfile, UserReq}, repository::user::UserRepository, state::AppState, utils::response::AppError};

#[allow(dead_code)]
pub struct UserService {
    user_repo: UserRepository,
    state: AppState
}

impl UserService {
    pub fn new(state: AppState) -> Self {
        let user_repo = UserRepository::new(state.database.clone());

        Self {user_repo, state}
    }

    pub async fn user_profile(&self, user_id: Uuid) -> Result<UserProfile, AppError> {
        let user = self.user_repo.find_user_profile(&user_id)
            .await.map_err(|e| AppError::NotFound(format!("User not found! {e}")))?;

        Ok(user)
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

    pub async fn user_update(&self, user: User, data: UpdateProfileReq) -> Result<UserProfile, AppError>{
        let _ = self.user_repo.update(user.id, data)
            .await.map_err(|e| AppError::BadRequest(format!{"Gagal update profile: {e}"}))?;

        let user_profile = self.user_repo.find_user_profile(&user.id).await?;
        Ok(user_profile)
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
