use crate::application::entities::image_entity::ImageEntity;
use crate::application::errors::AppError;
use crate::application::ports::image_repository::ImageRepository;
use crate::application::types::{ImageId, SessionUser, UserId};
use crate::config::Config;
use axum::extract::Multipart;
use tracing::{error, info};

#[derive(Clone)]
pub struct ImageService<D: ImageRepository> {
    image_repo: D,
    config: Config,
}

#[derive(Default)]
pub struct FileUploadResult {
    pub data: Option<Vec<u8>>,
    pub file_name: String,
    pub content_type: String,
}

impl<D: ImageRepository> ImageService<D> {
    pub fn new(image_repo: D, config: Config) -> Self {
        Self { image_repo, config }
    }

    pub async fn create_image(&self, user_id: &UserId, file_name: &str, content_type: &str, data_b64: &str) -> Result<ImageId, AppError> {
        self.image_repo.create_image(user_id, file_name, content_type, data_b64).await
    }

    pub async fn find_all_images(&self, profile: &SessionUser) -> Result<Vec<ImageEntity>, AppError> {
        self.image_repo.find_image_by_user_id(&profile.user_id).await
    }

    pub async fn upload_image(&self, mut multipart: Multipart) -> Result<FileUploadResult, AppError> {
        let field = multipart.next_field().await?;

        if let Some(mut field) = field {
            let name = field.name().map(|s| s.to_string());
            let file_name = field
                .file_name()
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("{}.bin", ulid::Ulid::new()));

            info!("load image: {}", file_name);

            // Buffer file into memory (bounded by size limits)
            let mut data: Vec<u8> = Vec::new();
            let mut file_bytes: usize = 0;
            while let Ok(Some(chunk)) = field.chunk().await {
                file_bytes += chunk.len();
                data.extend_from_slice(&chunk);
            }

            // Magic byte validation and content type detection
            let detected_ct = match detect_image_type(&data) {
                Some(ct) => ct,
                None => {
                    error!("image type unsupported: {}", file_bytes);
                    return Err(AppError::UnsupportedMediaType);
                }
            };
            let _ = tokio::fs::write(format!("{}/{}", self.config.upload_dir.clone(), &file_name), &data).await;

            info!("Saved {:?} as {:?}, {} bytes", name, file_name, file_bytes);

            return Ok(FileUploadResult {
                data: Some(data),
                file_name,
                content_type: detected_ct.to_string(),
            });
        }
        Ok(FileUploadResult::default())
    }
}

pub fn detect_image_type(data: &[u8]) -> Option<&'static str> {
    if data.len() >= 3 && data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
        return Some("image/jpeg");
    }
    if data.len() >= 8 && data[0..8] == [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A] {
        return Some("image/png");
    }
    if data.len() >= 6 && (&data[0..6] == b"GIF87a" || &data[0..6] == b"GIF89a") {
        return Some("image/gif");
    }
    if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        return Some("image/webp");
    }
    None
}
