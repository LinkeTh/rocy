use crate::application::errors::AppError;
use crate::application::ports::session_repository::SessionRepository;
use crate::application::types::{SessionId, SessionUser, UserId};
use async_trait::async_trait;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::time::SystemTime;

#[derive(Clone)]
pub struct PgSessionRepository {
    pub pool: PgPool,
}

#[async_trait]
impl SessionRepository for PgSessionRepository {
    async fn create_session(&self, user_id: &UserId, session_id: &SessionId, expires_at: SystemTime) -> Result<(), AppError> {
        let expires_at_chrono = DateTime::<Utc>::from(expires_at);

        sqlx::query("INSERT INTO sessions (user_id, session_id, expires_at) VALUES ($1, $2, $3)")
            .bind(user_id)
            .bind(session_id)
            .bind(expires_at_chrono)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn delete_session_by_id(&self, session_id: &SessionId) -> Result<(), AppError> {
        sqlx::query("DELETE FROM sessions WHERE session_id = $1")
            .bind(session_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn find_session_user_by_id(&self, session_id: &SessionId) -> Result<Option<SessionUser>, AppError> {
        let rec: Option<SessionUser> = sqlx::query_as(
            r#"SELECT users.id as user_id, users.email as email
               FROM sessions
               LEFT JOIN users ON sessions.user_id = users.id
               WHERE sessions.session_id = $1
               AND sessions.expires_at > now()
               LIMIT 1"#,
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(rec)
    }
}
