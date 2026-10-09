pub mod argparse;
pub mod camera;
pub mod com;
pub mod config;
pub mod data;
pub mod event_loop;
pub mod ml_engines;
pub mod test;

use crate::argparse::parse_arguments;
use crate::com::interface::ComInterface;
use crate::com::mqtt::ComMqtt;
use crate::config::parser::parse_config;
use crate::data::config::com::ComConfig;
use simple_logger::SimpleLogger;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if let Err(err) = SimpleLogger::new().init() {
        println! {"Failed to initialize logger: {err}"};
        Err(err.to_string())?
    };

    let args = parse_arguments();

    let config = parse_config(args.config_file)?;

    let mut com_interface: Box<dyn ComInterface> = match config.communication() {
        ComConfig::Opcua { opcua_config } => {
            todo!("OPC UA not implemented yet")
        }
        ComConfig::Mqtt { mqtt_config } => Box::new(ComMqtt::new(mqtt_config).await?),
    };

    Ok(())
}
