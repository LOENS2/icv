use crate::camera::interface::CameraInterface;
use crate::camera::sick_sec::auth::SecAuth;
use crate::data::config::SickSecConfig;
use async_trait::async_trait;
use std::error::Error;

pub struct Sec {
    sec_auth: SecAuth,
}

#[async_trait]
impl CameraInterface for Sec {
    async fn capture_image(&self) {
        todo!()
    }
}

impl Sec {
    pub async fn new(config: SickSecConfig) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            sec_auth: SecAuth::new(config).await?,
        })
    }
}
