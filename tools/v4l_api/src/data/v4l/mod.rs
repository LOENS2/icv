use image::DynamicImage;
use v4l::Format;

pub enum CaptureRequest {
    Capture {
        response: tokio::sync::oneshot::Sender<(Format, Vec<u8>)>,
    },
}
