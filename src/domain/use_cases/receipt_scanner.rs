use crate::application::errors::AppError;
use crate::application::ports::image_repository::ImageRepository;
use crate::application::ports::receipt_repository::ReceiptRepository;
use crate::application::types::UserId;
use crate::domain::receipt::{call_openai, ReceiptOcrResponse};
use crate::domain::services::image_service::ImageService;
use crate::domain::services::receipt_service::ReceiptService;
use axum::extract::Multipart;
use base64::Engine;
use std::sync::Arc;

#[derive(Clone)]
pub struct ReceiptScanner<S: ImageRepository, D: ReceiptRepository> {
    image_service: Arc<ImageService<S>>,
    receipt_service: Arc<ReceiptService<D>>,
}

impl<S: ImageRepository, D: ReceiptRepository> ReceiptScanner<S, D> {
    pub fn new(image_service: Arc<ImageService<S>>, receipt_service: Arc<ReceiptService<D>>) -> Self {
        Self {
            image_service,
            receipt_service,
        }
    }

    async fn scan_receipt(&self, b64: &str) -> Result<Option<ReceiptOcrResponse>, AppError> {
        // Base64 encode the image
        let result = call_openai(b64).await?;

        Ok(result)
    }

    pub async fn try_process_receipt(&self, multipart: Multipart, user_id: &UserId) -> Result<Option<String>, AppError> {
        let upload_result = self.image_service.upload_image(multipart).await?;

        match upload_result.data {
            Some(data) => {
                let b64 = base64::engine::general_purpose::STANDARD.encode(&data);

                let image_id = self
                    .image_service
                    .create_image(user_id, &upload_result.file_name, &upload_result.content_type, &b64)
                    .await?;

                // let events = self.events.publish(ImageUploadedEvent { image_id });;

                let scanned_receipt = self.scan_receipt(&b64).await?;
                let receipt_json = serde_json::to_string(&scanned_receipt)?;
                self.receipt_service.create_receipt(&image_id, &receipt_json).await?;

                Ok(Some(receipt_json))
            }
            None => Ok(None),
        }
    }
}
