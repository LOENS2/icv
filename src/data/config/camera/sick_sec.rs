use getset::Getters;
use serde::Deserialize;

#[derive(Deserialize, Getters)]
#[getset(get = "pub")]
pub struct SickSecConfig {
    host: String,
    username: String,
    password: String,
}
