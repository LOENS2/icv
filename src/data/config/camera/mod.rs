pub mod sick_sec;

use crate::data::config::camera::sick_sec::SickSecConfig;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum CameraConfig {
    SickSec {
        #[serde(rename = "sick_sec")]
        sick_sec_config: SickSecConfig,
    },
}
