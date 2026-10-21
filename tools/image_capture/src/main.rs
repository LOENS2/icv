mod config;

use ::config::{Config, File};
use clap::Parser;
use std::error::Error;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Arguments {
    #[arg(short, long, default_value = "")]
    pub config_file: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Arguments::parse();

    let config_file = args.config_file;

    let settings = Config::builder()
        .add_source(File::with_name(config_file.as_str()))
        .build()?;

    Ok(())
}
