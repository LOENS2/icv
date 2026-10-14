use async_trait::async_trait;
use image::DynamicImage;
use serde_json::Value;
use std::error::Error;

#[async_trait]
pub trait MlEngineInterface: Send + Sync {
    async fn predict(&self, image: DynamicImage) -> Result<Value, Box<dyn Error + Send + Sync>>;
}
