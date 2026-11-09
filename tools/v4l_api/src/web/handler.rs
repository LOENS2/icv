use crate::data::web::AppState;
use actix_web::{HttpResponse, Responder, get, web};
use image::ImageFormat::Png;
use std::error::Error;

#[get("/get-image")]
pub async fn get_image(data: web::Data<AppState>) -> Result<impl Responder, Box<dyn Error>> {
    let dynamic_image = data.v4l_capture.capture().await?;

    let mut png_data = Vec::new();

    dynamic_image.write_to(&mut std::io::Cursor::new(&mut png_data), Png)?;

    Ok(HttpResponse::Ok().content_type("image/png").body(png_data))
}
