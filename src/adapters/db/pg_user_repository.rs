use crate::application::entities::user_entity::UserEntity;
use crate::application::errors::AppError;
use crate::application::ports::user_repository::UserRepository;
use crate::application::types::{Email, Provider, Sub, UserId};
use async_trait::async_trait;
use sqlx::PgPool;

#[derive(Clone)]
pub struct PgUserRepository {
    pub pool: PgPool,
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn upsert_user(&self, provider: &Provider, sub: &Sub, email: &Email) -> Result<UserId, AppError> {
        let rec: UserId = sqlx::query_scalar(
            r#"INSERT INTO users (provider, sub, email)
               VALUES ($1, $2, $3)
               ON CONFLICT (provider, sub)
               DO UPDATE SET email = COALESCE(excluded.email, users.email)
               RETURNING id"#,
        )
        .bind(provider)
        .bind(sub)
        .bind(email)
        .fetch_one(&self.pool)
        .await?;
        Ok(rec)
    }

    async fn find_user_by_id(&self, user_id: &UserId) -> Result<Option<UserEntity>, AppError> {
        let rec: Option<UserEntity> = sqlx::query_as(
            r#"SELECT users.id as id,
               users.email as email,
               users.provider as provider,
               users.sub as sub,
               users.created_at as created
               FROM users
               WHERE users.id = $1
               LIMIT 1"#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(rec)
    }
}
