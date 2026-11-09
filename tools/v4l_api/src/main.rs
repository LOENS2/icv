use crate::argparse::parse_arguments;
use crate::config::load_config;
use crate::data::web::AppState;
use crate::v4l::V4lCapture;
use crate::web::handler::{get_frame_nv12, get_image};
use actix_web::{App, HttpServer, web::Data};
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Semaphore;

pub mod argparse;
pub mod config;
mod data;
pub mod v4l;
mod web;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if let Err(err) = SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .env()
        .init()
    {
        println! {"Failed to initialize logger: {err}"};
        Err(err.to_string())?
    };

    let args = parse_arguments();

    let config = load_config(args.config_file)?;

    let host = config.web().host();
    let port = config.web().port().to_owned();

    let v4l_capture = Arc::new(V4lCapture::new(config.v4l()).await);
    let capture_gate = Arc::new(Semaphore::new(1));

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState {
                v4l_capture: v4l_capture.clone(),
                capture_gate: capture_gate.clone(),
            }))
            .service(get_image)
            .service(get_frame_nv12)
    })
    .workers(1)
    .bind((host.as_str(), port))?
    .run()
    .await?;

    Ok(())
}
