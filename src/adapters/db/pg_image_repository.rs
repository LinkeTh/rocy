use crate::application::entities::image_entity::ImageEntity;
use crate::application::errors::AppError;
use crate::application::ports::image_repository::ImageRepository;
use crate::application::types::{ImageId, UserId};
use async_trait::async_trait;
use sqlx::PgPool;

#[derive(Clone)]
pub struct PgImageRepository {
    pub pool: PgPool,
}

#[async_trait]
impl ImageRepository for PgImageRepository {
    async fn create_image(&self, user_id: &UserId, file_name: &str, content_type: &str, data_b64: &str) -> Result<ImageId, AppError> {
        let id = sqlx::query_scalar::<_, ImageId>(
            "INSERT INTO user_images (user_id, file_name, content_type, data_base64) \
            VALUES ($1, $2, $3, $4) RETURNING id",
        )
        .bind(user_id)
        .bind(file_name)
        .bind(content_type)
        .bind(data_b64)
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
    }

    async fn find_image_by_user_id(&self, user_id: &UserId) -> Result<Vec<ImageEntity>, AppError> {
        let images = sqlx::query_as::<_, ImageEntity>(
            "SELECT id, user_id, file_name, content_type, data_base64
            FROM user_images
            WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(images)
    }
}
