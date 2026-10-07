pub mod camera;
pub mod comm;
pub mod ml_engines;

use crate::data::config::camera::CameraConfig;
use crate::data::config::comm::CommConfig;
use crate::data::config::ml_engines::MlEngineConfig;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct IcvConfig {
    communication: CommConfig,
    camera: CameraConfig,
    ml_engine: MlEngineConfig,
}
