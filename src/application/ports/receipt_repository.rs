use crate::application::errors::AppError;
use crate::application::types::{ImageId, ReceiptId};
use async_trait::async_trait;

#[async_trait]
pub trait ReceiptRepository: Send + Sync {
    async fn create(&self, image_id: &ImageId, json: &str) -> Result<ReceiptId, AppError>;
}
