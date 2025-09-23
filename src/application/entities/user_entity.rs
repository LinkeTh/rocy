use crate::application::types::{Email, Provider, Sub, UserId};
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};

#[derive(Deserialize, Serialize, sqlx::FromRow, Clone, sqlx::Type)]
pub struct UserEntity {
    pub id: UserId,
    pub provider: Provider,
    pub sub: Sub,
    pub email: Email,
    pub created: DateTime<Utc>,
}
