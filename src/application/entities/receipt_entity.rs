use crate::application::types::{ImageId, ReceiptId};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, sqlx::FromRow, Clone, sqlx::Type)]
pub struct ReceiptEntity {
    pub id: ReceiptId,
    pub image_id: ImageId,
    pub json: String,
}
