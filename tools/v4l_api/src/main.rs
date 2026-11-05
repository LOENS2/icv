use crate::argparse::parse_arguments;
use crate::config::load_config;
use crate::data::web::AppState;
use crate::web::handler::get_image;
use actix_web::{App, HttpServer, web::Data};
use std::error::Error;
use std::sync::Arc;
use v4l::Device;

pub mod argparse;
pub mod config;
mod data;
mod web;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = parse_arguments();

    let config = load_config(args.config_file)?;

    let host = config.web().host();
    let port = config.web().port().to_owned();

    let device_id = config.v4l().device_id().to_owned();
    let mut dev = Device::new(device_id)?;

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState {
                v4l_device: Arc::new(dev),
            }))
            .service(get_image)
    })
    .bind((host.as_str(), port))?
    .run()
    .await?;

    Ok(())
}
