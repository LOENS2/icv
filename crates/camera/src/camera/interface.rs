use async_trait::async_trait;
use image::DynamicImage;
use std::error::Error;

#[async_trait]
pub trait CameraInterface: Send + Sync {
    async fn capture_image(&self) -> Result<DynamicImage, Box<dyn Error + Send + Sync>>;
}
