use crate::argparse::parse_arguments;
use opcua::core::tracing::log;
use simple_logger::SimpleLogger;
use std::error::Error;
use std::process::exit;

pub mod argparse;
pub mod camera;
pub mod com;
pub mod config;
mod data;

fn main() -> Result<(), Box<dyn Error>> {
    if let Err(err) = SimpleLogger::new().init() {
        println! {"Failed to initialize logger: {err}"};
        Err(err.to_string())?
    };

    let args = parse_arguments();

    Ok(())
}
