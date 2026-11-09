use crate::v4l::V4lCapture;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub v4l_capture: Arc<V4lCapture>,
}
