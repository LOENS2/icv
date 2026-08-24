use async_trait::async_trait;

#[async_trait]
pub trait CameraInterface {
    async fn capture_image(&self);
}
