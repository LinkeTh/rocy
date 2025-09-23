use crate::application::entities::image_entity::ImageEntity;
use crate::application::errors::AppError;
use crate::application::types::{ImageId, UserId};
use async_trait::async_trait;

#[async_trait]
pub trait ImageRepository: Send + Sync {
    async fn create_image(&self, user_id: &UserId, file_name: &str, content_type: &str, data_b64: &str) -> Result<ImageId, AppError>;
    // async fn find_by_id(&self, id: &ImageId) -> Result<Option<String>, AppError>;
    async fn find_image_by_user_id(&self, user_id: &UserId) -> Result<Vec<ImageEntity>, AppError>;
    // async fn delete_by_id(&self, id: &ImageId) -> Result<(), AppError>;
    // async fn delete_by_user_id(&self, user_id: &UserId) -> Result<(), AppError>;
    // async fn delete_all(&self) -> Result<(), AppError>;
    // async fn count_by_user_id(&self, user_id: &UserId) -> Result<usize, AppError>;
    // async fn count_all(&self) -> Result<usize, AppError>;
    // async fn get_all(&self) -> Result<Vec<String>, AppError>;
    // async fn get_all_by_user_id(&self, user_id: &UserId) -> Result<Vec<String>, AppError>;
}
