use getset::Getters;
use serde::Deserialize;

#[derive(Deserialize, Getters)]
#[getset(get = "pub")]
pub struct OpcUaConfig {
    host: String,
    port: u16,
    username: Option<String>,
    password: Option<String>,
}
