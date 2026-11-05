use getset::Getters;
use serde::Deserialize;

#[derive(Deserialize, Getters)]
#[getset(get = "pub")]
pub struct WebConfig {
    host: String,
    port: u16,
    auth: String,
}
