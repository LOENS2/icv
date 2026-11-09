use crate::camera::http::Http;
use crate::camera::interface::CameraInterface;
use crate::camera::sick_sec::connection::Sec;
use crate::data::config::camera::CameraConfig;
use std::error::Error;
use std::sync::Arc;

pub mod camera;
pub mod data;
pub mod tests;

pub async fn create_from_config(
    config: CameraConfig,
) -> Result<Arc<dyn CameraInterface>, Box<dyn Error + Send + Sync>> {
    let camera_interface: Arc<dyn CameraInterface> = match config {
        CameraConfig::SickSec { sick_sec_config } => Arc::new(Sec::new(sick_sec_config).await?),
        CameraConfig::Http { url } => Arc::new(Http::new(url).await?),
    };

    Ok(camera_interface)
}
