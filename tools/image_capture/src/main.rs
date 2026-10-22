mod config;

use crate::config::ImageCaptureConfig;
use ::config::{Config, File};
use chrono::Local;
use clap::Parser;
use icv_camera::camera::interface::CameraInterface;
use icv_camera::camera::sick_sec::connection::Sec;
use icv_camera::data::config::camera::CameraConfig;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Arguments {
    #[arg(short, long, default_value = "")]
    pub config_file: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let args = Arguments::parse();

    let config_file = args.config_file;

    let settings = Config::builder()
        .add_source(File::with_name(config_file.as_str()))
        .build()?;

    let config = settings.try_deserialize::<ImageCaptureConfig>()?;

    let camera_interface: Arc<dyn CameraInterface> = match config.camera_config() {
        CameraConfig::SickSec { sick_sec_config } => Arc::new(Sec::new(sick_sec_config).await?),
    };

    let date_time_now = Local::now();
    let name_template = config.name_template();

    let filename = date_time_now.format(name_template.as_str()).to_string();
    let output_dir = PathBuf::from(config.output_dir());
    let output_path = output_dir.join(filename);

    let image = camera_interface.capture_image().await?;

    image.save(output_path)?;

    Ok(())
}
