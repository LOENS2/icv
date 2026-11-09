use getset::Getters;
use serde::Deserialize;

#[derive(Deserialize, Getters)]
#[getset(get = "pub")]
pub struct WebConfig {
    host: String,
    port: u16,
    auth_type: String,
    username: String,
    password: String,
}
