use crate::application::errors::AppError;
use crate::application::ports::session_repository::SessionRepository;
use crate::application::types::{SessionId, SessionUser, UserId};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::Rng;
use std::time::SystemTime;

#[derive(Clone)]
pub struct SessionService<S>
where
    S: SessionRepository,
{
    session_repo: S,
}
impl<S> SessionService<S>
where
    S: SessionRepository,
{
    pub fn new(session_repo: S) -> Self {
        Self { session_repo }
    }

    pub async fn find_user_by_session(&self, session_id: &SessionId) -> Result<Option<SessionUser>, AppError> {
        self.session_repo.find_session_user_by_id(session_id).await
    }

    pub async fn delete_session(&self, session_id: &SessionId) -> Result<(), AppError> {
        self.session_repo.delete_session_by_id(session_id).await
    }

    pub async fn create_session(&self, user_id: &UserId, session_id: &SessionId, expires_at: SystemTime) -> Result<(), AppError> {
        self.session_repo.create_session(user_id, session_id, expires_at).await
    }
}

pub fn generate_session_id() -> SessionId {
    let mut sid_bytes = [0u8; 32];
    rand::rng().fill(&mut sid_bytes);
    let sid = URL_SAFE_NO_PAD.encode(sid_bytes);
    SessionId(sid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::entities::session_entity::SessionEntity;
    use crate::application::types::UserId;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[derive(Clone, Default)]
    struct InMemorySessionRepo {
        sessions: Arc<Mutex<HashMap<SessionId, SessionEntity>>>,
    }

    impl InMemorySessionRepo {
        fn new() -> Self {
            Self::default()
        }
    }

    #[async_trait]
    impl SessionRepository for InMemorySessionRepo {
        async fn create_session(&self, user_id: &UserId, session_id: &SessionId, expires_at: SystemTime) -> Result<(), AppError> {
            let expires_at_chrono = sqlx::types::chrono::DateTime::<sqlx::types::chrono::Utc>::from(expires_at);

            self.sessions.lock().unwrap().insert(
                session_id.clone(),
                SessionEntity {
                    session_id: session_id.clone(),
                    user_id: user_id.clone(),
                    expires_at: expires_at_chrono,
                },
            );
            Ok(())
        }

        async fn delete_session_by_id(&self, session_id: &SessionId) -> Result<(), AppError> {
            let _ = self.sessions.lock().unwrap().remove(&session_id);
            Ok(())
        }

        async fn find_session_user_by_id(&self, session_id: &SessionId) -> Result<Option<SessionUser>, AppError> {
            let session_entity = self.sessions.lock().unwrap().get(&session_id).cloned();

            let session_user: Option<SessionUser> = session_entity.map(|e| SessionUser {
                user_id: e.user_id,
                email: crate::application::types::Email("test@example.com".into()),
            });
            Ok(session_user)
        }
    }

    #[tokio::test]
    async fn create_and_find_user_by_session_id_returns_user() {
        // Arrange
        let repo = InMemorySessionRepo::new();
        let service = SessionService::new(repo.clone());

        let user_id = UserId(1);
        let session_id = generate_session_id();
        let expires_at = SystemTime::now() + Duration::from_secs(60);

        // Act
        service.create_session(&user_id, &session_id, expires_at).await.unwrap();
        let found = service.find_user_by_session(&session_id).await.unwrap();

        // Assert
        let user = found.expect("expected Some(SessionUser)");
        assert_eq!(user.user_id, user_id);
        assert_eq!(user.email.0, "test@example.com");
    }

    #[tokio::test]
    async fn find_user_by_session_id_unknown_returns_none() {
        // Arrange
        let repo = InMemorySessionRepo::new();
        let service = SessionService::new(repo);

        let unknown_sid = generate_session_id();

        // Act
        let found = service.find_user_by_session(&unknown_sid).await.unwrap();

        // Assert
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn delete_by_id_removes_session() {
        // Arrange
        let repo = InMemorySessionRepo::new();
        let service = SessionService::new(repo.clone());

        let user_id = UserId(1);
        let session_id = generate_session_id();
        let expires_at = SystemTime::now() + Duration::from_secs(60);

        service.create_session(&user_id, &session_id, expires_at).await.unwrap();

        // Sanity check present
        assert!(service.find_user_by_session(&session_id).await.unwrap().is_some());

        // Act
        service.delete_session(&session_id).await.unwrap();

        // Assert
        assert!(service.find_user_by_session(&session_id).await.unwrap().is_none());
    }
}
