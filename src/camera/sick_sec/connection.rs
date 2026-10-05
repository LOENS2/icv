use crate::camera::interface::CameraInterface;
use crate::camera::sick_sec::auth::{ResponseType, SecAuth};
use crate::data::config::camera::sick_sec::SickSecConfig;
use async_trait::async_trait;
use image::{DynamicImage, ImageReader};
use serde_json::json;
use std::error::Error;
use std::io::Cursor;
use uuid::Uuid;

pub struct Sec {
    sec_auth: SecAuth,
}

#[async_trait]
impl CameraInterface for Sec {
    async fn capture_image(&self) -> Result<DynamicImage, Box<dyn Error>> {
        const DOWNLOAD_FILE_PATH: &str = "latestSnapshot";
        let snapshot_name = Uuid::new_v4().to_string();

        self.trigger_named_snapshot(&snapshot_name).await?;

        let image_data = match self.download_file(DOWNLOAD_FILE_PATH).await? {
            ResponseType::Jpeg { data } => ImageReader::new(Cursor::new(data))
                .with_guessed_format()?
                .decode()?,
            _ => return Err("Downloaded file is not a JPEG. This is an internal bug.".into()),
        };

        let file_name = format!("{snapshot_name}.jpeg");

        self.delete_file(&file_name).await?;

        Ok(image_data)
    }
}

impl Sec {
    pub async fn new(config: SickSecConfig) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            sec_auth: SecAuth::new(config).await?,
        })
    }

    pub async fn check_credentials(&self) -> Result<(), Box<dyn Error>> {
        const CHECK_CREDENTIALS_PATH: &str = "/api/checkCredentials";

        self.sec_auth.post(CHECK_CREDENTIALS_PATH, None).await?;

        Ok(())
    }

    pub async fn trigger_named_snapshot(&self, name: &String) -> Result<(), Box<dyn Error>> {
        const TRIGGER_NAMED_SNAPSHOT_PATH: &str = "/api/SnapshotTriggerNamedSnapshot";

        let post_data = json!({
            "SnapshotName": name
        });

        self.sec_auth
            .post(TRIGGER_NAMED_SNAPSHOT_PATH, Some(post_data))
            .await?;

        Ok(())
    }

    pub async fn download_file(&self, file_name: &str) -> Result<ResponseType, Box<dyn Error>> {
        let download_file_path = format!("/file/download/{file_name}");

        let response_data = self
            .sec_auth
            .post(download_file_path.as_str(), None)
            .await?;

        match response_data {
            ResponseType::Json { data: _ } => {
                Err("File cannot be json, this is an internal bug!".into())
            }
            _ => Ok(response_data),
        }
    }

    pub async fn delete_file(&self, file_name: &String) -> Result<(), Box<dyn Error>> {
        const DELETE_FILE_PATH: &str = "/api/DeleteFile";

        let post_data = json!({
            "fileName": file_name
        });

        self.sec_auth
            .post(DELETE_FILE_PATH, Some(post_data))
            .await?;

        Ok(())
    }
}
