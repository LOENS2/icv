use async_trait::async_trait;
use image::DynamicImage;
use serde_json::Value;
use std::error::Error;

#[async_trait]
pub trait MlEngineInterface {
    async fn predict(&mut self, image: DynamicImage) -> Result<Value, Box<dyn Error>>;
}
