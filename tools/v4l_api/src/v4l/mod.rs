mod frame;
mod mplane_mmap;

pub use frame::Frame;

use crate::data::config::v4l::V4lConfig;
use crate::data::v4l::CaptureRequest;
use log::error;
use std::error::Error;
use tokio::runtime::Builder;
use tokio::sync::{mpsc, oneshot};
use tokio::task;

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

    pub async fn capture_frame(&self) -> Result<Frame, Box<dyn Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();

        self.to_thread_tx
            .send(CaptureRequest::Capture { response: tx })
            .await
            .map_err(|e| -> Box<dyn Error + Send + Sync> {
                format!("capture channel closed: {e}").into()
            })?;

        let (format, buf) = rx.await.map_err(|e| -> Box<dyn Error + Send + Sync> {
            format!("capture response dropped: {e}").into()
        })?;

        Frame::from_capture(format, buf)
    }
}
