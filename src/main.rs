use crate::argparse::parse_arguments;
use simple_logger::SimpleLogger;
use std::error::Error;

pub mod argparse;
pub mod camera;
pub mod com;
pub mod config;
pub mod data;
pub mod test;

fn main() -> Result<(), Box<dyn Error>> {
    if let Err(err) = SimpleLogger::new().init() {
        println! {"Failed to initialize logger: {err}"};
        Err(err.to_string())?
    };

    let _args = parse_arguments();

    Ok(())
}
