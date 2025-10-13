use crate::application::errors::AppError;
use crate::application::ports::receipt_repository::ReceiptRepository;
use crate::application::types::{ImageId, ReceiptId};

#[derive(Clone)]
pub struct ReceiptService<D: ReceiptRepository> {
    receipt_repo: D,
}

#[derive(Clone, Debug)]
pub struct ImageUploadedEvent {
    image_id: ImageId,
}

impl<D: ReceiptRepository> ReceiptService<D> {
    pub fn new(receipt_repo: D) -> Self {
        Self { receipt_repo }
    }

    pub async fn create_receipt(&self, image_id: &ImageId, json: &str) -> Result<ReceiptId, AppError> {
        self.receipt_repo.create(image_id, json).await
    }
}
