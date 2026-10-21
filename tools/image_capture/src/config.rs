use getset::CloneGetters;
use icv_camera::data::config::camera::CameraConfig;
use serde::Deserialize;

#[derive(Deserialize, CloneGetters)]
#[getset(get_clone = "pub")]
pub struct ImageCaptureConfig {
    output_dir: String,
    name_template: String,
    camera_config: CameraConfig,
}
