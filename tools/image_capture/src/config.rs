use getset::Getters;
use icv_camera::data::config::camera::CameraConfig;
use serde::Deserialize;

#[derive(Deserialize, Getters)]
#[getset(get = "pub")]
struct Config {
    output_dir: String,
    name_template: String,
    camera_config: CameraConfig,
}
