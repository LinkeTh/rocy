use crate::application::entities::user_entity::UserEntity;
use crate::application::errors::AppError;
use crate::application::types::{Email, Provider, Sub, UserId};
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn upsert_user(&self, provider: &Provider, sub: &Sub, email: &Email) -> Result<UserId, AppError>;
    async fn find_user_by_id(&self, user_id: &UserId) -> Result<Option<UserEntity>, AppError>;
}
