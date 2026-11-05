use std::sync::Arc;

pub struct AppState {
    pub v4l_device: Arc<v4l::Device>,
}
