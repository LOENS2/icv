use getset::CopyGetters;
use serde::Deserialize;

#[derive(Deserialize, CopyGetters, Clone)]
#[getset(get_copy = "pub")]
pub struct HttpConfig {
    url: String,
}
