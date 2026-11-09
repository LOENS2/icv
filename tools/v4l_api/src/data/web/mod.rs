use crate::v4l::V4lCapture;
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct AppState {
    pub v4l_capture: Arc<V4lCapture>,
    pub capture_gate: Arc<Semaphore>,
}
