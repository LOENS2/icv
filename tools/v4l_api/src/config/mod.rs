use crate::data::config::V4lApiConfig;
use config::{Config, Environment, File};
use std::error::Error;

pub fn load_config(config_file: String) -> Result<V4lApiConfig, Box<dyn Error>> {
    let settings = Config::builder()
        .add_source(File::with_name(config_file.as_str()))
        .add_source(Environment::with_prefix("ICV"))
        .build()?;

    Ok(settings.try_deserialize::<V4lApiConfig>()?)
}
