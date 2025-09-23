use crate::application::types::{ImageId, UserId};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, sqlx::FromRow, Clone, sqlx::Type)]
pub struct ImageEntity {
    pub id: ImageId,
    pub user_id: UserId,
    pub file_name: String,
    pub content_type: String,
    pub data_base64: String,
}
