mod mplane_mmap;

use crate::data::config::v4l::V4lConfig;
use crate::data::v4l::CaptureRequest;
use image::{DynamicImage, RgbImage};
use log::{debug, error};
use std::error::Error;
use tokio::runtime::Builder;
use tokio::sync::{mpsc, oneshot};
use tokio::task;
use tokio_linux_video::types::{FourCc, PixFormatMplane};
use yuvutils_rs::{
    YuvBiPlanarImage, YuvConversionMode, YuvRange, YuvStandardMatrix, yuv_nv12_to_rgb,
};

#[derive(Clone)]
pub struct V4lCapture {
    to_thread_tx: mpsc::Sender<CaptureRequest>,
}

impl V4lCapture {
    pub async fn new(config: &V4lConfig) -> Self {
        let device_path = config.device_path().clone();

        let (to_thread_tx, to_thread_rx) = mpsc::channel::<CaptureRequest>(8);

        task::spawn_blocking(move || {
            let rt = match Builder::new_current_thread().enable_all().build() {
                Ok(rt) => rt,
                Err(e) => {
                    error!("Failed to build capture runtime: {e}");
                    return;
                }
            };

            if let Err(e) = rt.block_on(mplane_mmap::capture_thread(&device_path, to_thread_rx)) {
                error!("Error in capture thread: {e}");
            }
        });

        V4lCapture { to_thread_tx }
    }

    pub async fn capture(&self) -> Result<DynamicImage, Box<dyn Error>> {
        let (tx, rx) = oneshot::channel();

        self.to_thread_tx
            .send(CaptureRequest::Capture { response: tx })
            .await?;

        let (format, buf) = rx.await?;

        let pix = format
            .try_ref::<PixFormatMplane>()
            .ok_or("Expected multi-planar video format")?;

        let width = pix.width();
        let height = pix.height();
        let pixel_format = pix.pixel_format();
        let planes = pix.plane_fmt();

        let (y_stride, uv_stride, y_plane, uv_plane) = match planes {
            [y, uv, ..] => {
                let y_stride = y.bytes_per_line();
                let uv_stride = uv.bytes_per_line();
                let y_size = y.size_image().max(y_stride * height) as usize;
                let uv_size = uv.size_image().max(uv_stride * (height / 2)) as usize;

                if buf.len() < y_size + uv_size {
                    return Err(format!(
                        "Buffer too small: expected at least {}, got {}",
                        y_size + uv_size,
                        buf.len()
                    )
                    .into());
                }

                let (y_plane, rest) = buf.split_at(y_size);
                let uv_plane = &rest[..uv_size];
                (y_stride, uv_stride, y_plane, uv_plane)
            }
            [y] => {
                let y_stride = y.bytes_per_line();
                let uv_stride = y_stride;
                let y_size = (y_stride * height) as usize;
                let uv_size = y_size / 2;
                let expected_size = y_size + uv_size;

                if buf.len() < expected_size {
                    return Err(format!(
                        "Buffer too small: expected at least {}, got {}",
                        expected_size,
                        buf.len()
                    )
                    .into());
                }

                let (y_plane, rest) = buf.split_at(y_size);
                let uv_plane = &rest[..uv_size];
                (y_stride, uv_stride, y_plane, uv_plane)
            }
            _ => return Err("No plane format information available".into()),
        };

        debug!(
            "width: {}, height: {}, y_stride: {}, uv_stride: {}, format: {}",
            width, height, y_stride, uv_stride, pixel_format
        );

        if pixel_format != FourCc::Nv12 && pixel_format != FourCc::Nv21 {
            return Err(format!("Unsupported format: {pixel_format} (expected NV12/NV21)").into());
        }

        if pixel_format == FourCc::Nv21 {
            return Err("NV21 is not supported yet (expected NV12)".into());
        }

        let yuv_image = YuvBiPlanarImage {
            y_plane,
            y_stride,
            uv_plane,
            uv_stride,
            width,
            height,
        };

        let rgb_stride = width * 3;
        let mut data_rgb = vec![0u8; (rgb_stride * height) as usize];

        yuv_nv12_to_rgb(
            &yuv_image,
            data_rgb.as_mut_slice(),
            rgb_stride,
            YuvRange::Full,
            YuvStandardMatrix::Bt709,
            YuvConversionMode::Balanced,
        )?;

        let rgb_image =
            RgbImage::from_raw(width, height, data_rgb).ok_or("Failed to create RGB image")?;

        Ok(DynamicImage::ImageRgb8(rgb_image))
    }
}
