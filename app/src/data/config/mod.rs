pub mod com;
pub mod ml_engines;

use crate::data::config::com::ComConfig;
use crate::data::config::ml_engines::MlEngineConfig;
use getset::CloneGetters;
use icv_camera::data::config::camera::CameraConfig;
use serde::Deserialize;

#[derive(Deserialize, CloneGetters)]
#[getset(get_clone = "pub")]
pub struct IcvConfig {
    communication: ComConfig,
    camera: CameraConfig,
    ml_engine: MlEngineConfig,
}
