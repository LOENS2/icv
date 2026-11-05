use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Arguments {
    #[arg(short, long, default_value = "")]
    pub config_file: String,
}

pub fn parse_arguments() -> Arguments {
    Arguments::parse()
}
