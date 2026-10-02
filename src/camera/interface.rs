use async_trait::async_trait;
use bytes::Bytes;
use std::error::Error;

#[async_trait]
pub trait CameraInterface {
    async fn capture_image(&self) -> Result<Bytes, Box<dyn Error>>;
}
