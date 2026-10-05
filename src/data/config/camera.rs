use getset::Getters;
use serde::Deserialize;

#[derive(Deserialize, Getters)]
#[getset(get = "pub")]
pub struct SickSecConfig {
    host: String,
    username: String,
    password: String,
}

#[derive(Deserialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum CameraConfig {
    SickSec {
        #[serde(rename = "sick_sec")]
        sick_sec_config: SickSecConfig,
    },
}
