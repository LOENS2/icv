use getset::Getters;
use serde::Deserialize;

#[derive(Deserialize, Getters)]
#[getset(get = "pub")]
pub struct V4lConfig {
    device_id: usize,
}
