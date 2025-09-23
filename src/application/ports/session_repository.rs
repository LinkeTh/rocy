use crate::application::errors::AppError;
use crate::application::types::{SessionId, SessionUser, UserId};
use async_trait::async_trait;
use std::time::SystemTime;

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn create_session(&self, user_id: &UserId, session_id: &SessionId, expires_at: SystemTime) -> Result<(), AppError>;

    async fn delete_session_by_id(&self, session_id: &SessionId) -> Result<(), AppError>;

    async fn find_session_user_by_id(&self, session_id: &SessionId) -> Result<Option<SessionUser>, AppError>;
}
