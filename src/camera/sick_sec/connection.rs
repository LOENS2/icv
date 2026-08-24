use crate::camera::interface::CameraInterface;

struct Sec {}

impl CameraInterface for Sec {
    async fn capture_image(&self) {
        todo!()
    }
}

impl Sec {
    pub fn new() -> Self {
        Self {}
    }
}
