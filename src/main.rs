pub mod argparse;
pub mod camera;
pub mod com;
pub mod config;
pub mod data;
pub mod event_loop;
pub mod ml_engines;
pub mod test;

use crate::argparse::parse_arguments;
use crate::camera::interface::CameraInterface;
use crate::camera::sick_sec::connection::Sec;
use crate::com::interface::ComInterface;
use crate::com::mqtt::ComMqtt;
use crate::config::parser::parse_config;
use crate::data::config::camera::CameraConfig;
use crate::data::config::com::ComConfig;
use crate::data::config::ml_engines::MlEngineBackend;
use crate::event_loop::run_app;
use crate::ml_engines::interface::MlEngineInterface;
use crate::ml_engines::onnx::OrtEngine;
use simple_logger::SimpleLogger;
use std::error::Error;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Err(err) = SimpleLogger::new().init() {
        println! {"Failed to initialize logger: {err}"};
        Err(err.to_string())?
    };

    let args = parse_arguments();

    let config = parse_config(args.config_file)?;

    let camera_interface: Arc<dyn CameraInterface> = match config.camera() {
        CameraConfig::SickSec { sick_sec_config } => Arc::new(Sec::new(sick_sec_config).await?),
    };

    let ml_engine_config = config.ml_engine();

    let ml_engine: Arc<dyn MlEngineInterface> = match ml_engine_config.backend() {
        MlEngineBackend::Ort { ort_config } => {
            Arc::new(OrtEngine::new(ort_config, ml_engine_config.model_dir()).await?)
        }
        MlEngineBackend::HailoRT => {
            todo!("HailoRT is not implemented yet.")
        }
    };

    let com_interface: Box<dyn ComInterface> = match config.communication() {
        ComConfig::OpcUa { opcua_config } => {
            todo!("OPC UA not implemented yet {}", opcua_config.host())
        }
        ComConfig::Mqtt { mqtt_config } => Box::new(ComMqtt::new(mqtt_config).await?),
    };

    run_app(camera_interface, ml_engine, com_interface).await?;

    Ok(())
}
