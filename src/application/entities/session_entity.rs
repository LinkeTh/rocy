use crate::application::types::{SessionId, UserId};
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};

#[derive(Deserialize, Serialize, sqlx::FromRow, Clone, sqlx::Type)]
pub struct SessionEntity {
    pub session_id: SessionId,
    pub user_id: UserId,
    pub expires_at: DateTime<Utc>,
}
