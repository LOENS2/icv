use crate::camera::interface::CameraInterface;
use async_trait::async_trait;
use image::{DynamicImage, ImageReader};
use std::error::Error;
use std::io::Cursor;

pub struct Http {
    url: String,
}

#[async_trait]
impl CameraInterface for Http {
    async fn capture_image(&self) -> Result<DynamicImage, Box<dyn Error + Send + Sync>> {
        let response = reqwest::get(&self.url).await?;

        let data = response.bytes().await?;

        let dynamic_image = ImageReader::new(Cursor::new(data))
            .with_guessed_format()?
            .decode()?;

        Ok(dynamic_image)
    }
}

impl Http {
    pub async fn new(url: &String) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(Self {
            url: url.to_owned(),
        })
    }
}
