use getset::Getters;
use serde::Deserialize;

#[derive(Deserialize, Getters, Clone)]
#[getset(get = "pub")]
pub struct HttpConfig {
    url: String,
}
