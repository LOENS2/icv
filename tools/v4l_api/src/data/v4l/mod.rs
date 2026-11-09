use tokio_linux_video::types::Format;

pub enum CaptureRequest {
    Capture {
        response: tokio::sync::oneshot::Sender<(Format, Vec<u8>)>,
    },
}
