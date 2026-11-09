use crate::data::config::v4l::V4lConfig;
use crate::data::v4l::CaptureRequest;
use image::{DynamicImage, RgbImage};
use log::{debug, error};
use std::error::Error;
use tokio::sync::{mpsc, oneshot};
use v4l::Device;
use v4l::buffer::Type;
use v4l::io::traits::CaptureStream;
use v4l::prelude::MmapStream;
use v4l::video::Capture;
use yuvutils_rs::{
    YuvBiPlanarImage, YuvConversionMode, YuvRange, YuvStandardMatrix, yuv_nv12_to_rgb,
};

#[derive(Clone)]
pub struct V4lCapture {
    to_thread_tx: mpsc::Sender<CaptureRequest>,
}

impl V4lCapture {
    pub async fn new(config: &V4lConfig) -> Self {
        let device_id = config.device_id().to_owned();

        let (to_thread_tx, to_thread_rx) = mpsc::channel::<CaptureRequest>(8);

        tokio::spawn(async move {
            if let Err(e) = Self::capture_thread(device_id, to_thread_rx).await {
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

        let width = format.width;
        let height = format.height;
        let stride = format.stride;

        debug!("width: {}, height: {}", width, height);
        debug!("Format: {:?}", format);

        if format.fourcc.str()? != "NV12" {
            return Err(format!("Unsupported format: {:?}", format).into());
        }

        let y_size = stride * height;
        let uv_size = y_size / 2;

        let expected_size = (y_size + uv_size) as usize;

        if buf.len() < expected_size {
            return Err(format!(
                "Buffer too small: expected {}, got {}",
                expected_size,
                buf.len()
            )
            .into());
        }

        let (y_plane, uv_plane) = buf.split_at(y_size as usize);

        if uv_plane.len() != uv_size as usize {
            return Err("Invalid NV12 buffer size".into());
        }

        let yuv_image = YuvBiPlanarImage {
            y_plane,
            y_stride: stride,
            uv_plane,
            uv_stride: stride,
            width,
            height,
        };
        let range = YuvRange::Full;
        let matrix = YuvStandardMatrix::Bt709;
        let mode = YuvConversionMode::Balanced;

        let rgb_stride = width * 3;
        let mut data_rgb = vec![0u8; (rgb_stride * height) as usize];

        yuv_nv12_to_rgb(
            &yuv_image,
            data_rgb.as_mut_slice(),
            rgb_stride,
            range,
            matrix,
            mode,
        )?;

        let rgb_image = match RgbImage::from_raw(width, height, data_rgb) {
            Some(image) => image,
            None => {
                return Err("Failed to create RGB image".into());
            }
        };

        let dynamic_image = DynamicImage::ImageRgb8(rgb_image);

        Ok(dynamic_image)
    }

    async fn capture_thread(
        device_id: usize,
        mut rx: mpsc::Receiver<CaptureRequest>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut device = Device::new(device_id)?;
        let format = device.format()?;
        let mut stream = MmapStream::with_buffers(&mut device, Type::VideoCapture, 4)?;

        while let Some(request) = rx.recv().await {
            match request {
                CaptureRequest::Capture { response } => {
                    let (buf, _) = stream.next()?;

                    let _ = response.send((format.clone(), buf.to_vec()));
                }
            }
        }

        Ok(())
    }
}
