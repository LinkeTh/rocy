use crate::application::errors::AppError;
use crate::application::ports::receipt_repository::ReceiptRepository;
use crate::application::types::{ImageId, ReceiptId};
use async_trait::async_trait;
use sqlx::PgPool;

#[derive(Clone)]
pub struct PgReceiptRepository {
    pub pool: PgPool,
}

#[async_trait]
impl ReceiptRepository for PgReceiptRepository {
    async fn create(&self, image_id: &ImageId, json: &str) -> Result<ReceiptId, AppError> {
        let id = sqlx::query_scalar::<_, ReceiptId>("INSERT INTO user_receipts (image_id, json) VALUES ($1, $2) RETURNING id")
            .bind(image_id)
            .bind(json)
            .fetch_one(&self.pool)
            .await?;
        Ok(id)
    }
}
