use crate::data::config::v4l::V4lConfig;
use crate::data::config::web::WebConfig;
use getset::Getters;
use serde::Deserialize;

pub mod v4l;
pub mod web;

#[derive(Deserialize, Getters)]
#[getset(get = "pub")]
pub struct V4lApiConfig {
    web: WebConfig,
    v4l: V4lConfig,
}
