use crate::data::web::AppState;
use actix_web::{HttpResponse, Responder, get, web};
use image::ImageFormat::Png;
use image::{DynamicImage, ImageFormat, RgbImage};
use v4l::buffer::Type;
use v4l::io::traits::OutputStream;
use v4l::prelude::MmapStream;
use yuvutils_rs::{
    YuvBiPlanarImage, YuvConversionMode, YuvPlanarImage, YuvRange, YuvStandardMatrix,
    yuv_nv12_to_rgb,
};

#[get("/get-image")]
pub async fn get_image(data: web::Data<AppState>) -> impl Responder {
    let mut device = &data.v4l_device;

    let mut stream = MmapStream::with_buffers(&mut device, Type::VideoCapture, 1)?;

    let (mut buf, meta) = stream.next()?;

    let width: usize = 0;
    let height: usize = 0;

    let yuv_image = YuvPlanarImage::from(buf);
    let mut data_rgb = Vec::new();
    let rgb_stride = 0;
    let range = YuvRange::Full;
    let matrix = YuvStandardMatrix::Bt709;
    let mode = YuvConversionMode::Balanced;

    yuv_nv12_to_rgb(
        buf,
        data_rgb.as_mut_slice(),
        rgb_stride,
        range,
        matrix,
        mode,
    )?;

    let rgb_image = RgbImage::from_raw(width as u32, height as u32, data_rgb)?;

    let dynamic_image = DynamicImage::ImageRgb8(rgb_image);

    let png = dynamic_image.write_with_encoder(ImageFormat::Png);

    HttpResponse::Ok().content_type("image/png").body()
}
