pub mod camera;
pub mod com;
pub mod ml_engines;

use crate::data::config::camera::CameraConfig;
use crate::data::config::com::ComConfig;
use crate::data::config::ml_engines::MlEngineConfig;
use getset::Getters;
use serde::Deserialize;

#[derive(Deserialize, Getters)]
#[getset(get = "pub")]
pub struct IcvConfig {
    communication: ComConfig,
    camera: CameraConfig,
    ml_engine: MlEngineConfig,
}
