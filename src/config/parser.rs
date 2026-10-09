use crate::data::config::IcvConfig;
use config::{Config, Environment, File};
use std::error::Error;

pub fn parse_config(config_file: String) -> Result<IcvConfig, Box<dyn Error>> {
    let settings = Config::builder()
        .add_source(File::with_name(config_file.as_str()))
        .add_source(Environment::with_prefix("ICV"))
        .build()?;

    Ok(settings.try_deserialize::<IcvConfig>()?)
}
