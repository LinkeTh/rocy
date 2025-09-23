use crate::application::entities::image_entity::ImageEntity;
use crate::application::errors::AppError;
use crate::application::ports::image_repository::ImageRepository;
use crate::application::types::{ImageId, SessionUser, UserId};
use crate::config::Config;
use crate::domain;
use axum::extract::Multipart;
use base64::Engine;
use serde_json::Value;
use tracing::error;

#[derive(Clone)]
pub struct ImageService<D: ImageRepository> {
    image_repo: D,
    config: Config,
}

impl<D: ImageRepository> ImageService<D> {
    pub fn new(image_repo: D, config: Config) -> Self {
        Self { image_repo, config }
    }

    pub async fn store(&self, user_id: &UserId, file_name: &str, content_type: &str, data_b64: &str) -> Result<ImageId, AppError> {
        self.image_repo.create_image(user_id, file_name, content_type, data_b64).await
    }

    pub async fn find_all_images(&self, profile: SessionUser) -> Result<Vec<ImageEntity>, AppError> {
        self.image_repo.find_image_by_user_id(&profile.user_id).await
    }

    pub async fn upload_image(&self, mut multipart: Multipart, profile: SessionUser) -> Result<Vec<Value>, AppError> {
        let mut saved_files = Vec::new();
        let mut total_bytes: usize = 0;

        while let Ok(Some(mut field)) = multipart.next_field().await {
            let name = field.name().map(|s| s.to_string());
            let file_name = field
                .file_name()
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("{}.bin", ulid::Ulid::new()));

            // Buffer file into memory (bounded by size limits)
            let mut data: Vec<u8> = Vec::new();
            let mut file_bytes: usize = 0;
            while let Ok(Some(chunk)) = field.chunk().await {
                file_bytes += chunk.len();
                total_bytes += chunk.len();
                if file_bytes > self.config.per_file_max_bytes || total_bytes > self.config.max_upload_bytes {
                    return Err(AppError::PayloadTooLarge);
                }
                data.extend_from_slice(&chunk);
            }

            // Magic byte validation and content type detection
            let detected_ct = match domain::image::detect_image_type(&data) {
                Some(ct) => ct,
                None => {
                    return Err(AppError::UnsupportedMediaType);
                }
            };

            // Base64 encode the image
            let b64 = base64::engine::general_purpose::STANDARD.encode(&data);

            // Store into DB via repository adapter
            let image_id: ImageId = match self.store(&profile.user_id, &file_name, detected_ct, &b64).await {
                Ok(id) => id,
                Err(e) => {
                    error!(?e, "failed to insert user image");
                    return Err(e);
                }
            };

            saved_files.push(serde_json::json!({
            "field": name,
            "file_name": file_name,
            "content_type": detected_ct,
            "size": file_bytes,
            "id": image_id
            }));
        }
        Ok(saved_files)
    }
}
