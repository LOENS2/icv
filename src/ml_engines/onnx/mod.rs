use crate::data::config::ml_engines::ort::OrtConfig;
use crate::ml_engines::interface::MlEngineInterface;
use async_trait::async_trait;
use image::DynamicImage;
use image::imageops::FilterType;
use ort::ep;
use ort::session::{RunOptions, Session};
use ort::value::Tensor;
use serde_json::Value;
use std::error::Error;
use std::path::PathBuf;
use tokio::sync::Mutex;

pub struct OrtEngine {
    config: OrtConfig,
    session: Mutex<Session>,
}

#[async_trait]
impl MlEngineInterface for OrtEngine {
    async fn predict(&self, image: DynamicImage) -> Result<Value, Box<dyn Error + Send + Sync>> {
        const TARGET_WIDTH: u32 = 1024;
        const TARGET_HEIGHT: u32 = 1024;

        let resized_image = image.resize(TARGET_HEIGHT, TARGET_WIDTH, FilterType::Triangle);

        let resized_image_rgb = match resized_image.as_rgb8() {
            Some(image_rgb) => image_rgb,
            None => return Err("No rgb image".into()),
        };

        let pixel_count = (TARGET_WIDTH * TARGET_HEIGHT) as usize;

        let mut resized_image_array = vec![0.0f32; 3 * pixel_count];

        let (r_slice, rest) = resized_image_array.split_at_mut(pixel_count);
        let (g_slice, b_slice) = rest.split_at_mut(pixel_count);

        for (i, pixel) in resized_image_rgb.pixels().enumerate() {
            r_slice[i] = pixel[0] as f32 / 255.0;
            g_slice[i] = pixel[1] as f32 / 255.0;
            b_slice[i] = pixel[2] as f32 / 255.0;
        }

        let mut session = self.session.lock().await;

        let input_name = session.inputs()[0].name().to_owned();
        let output_name = session.outputs()[0].name().to_owned();
        let input_tensor = Tensor::from_array((
            [1, 3, TARGET_HEIGHT as i64, TARGET_WIDTH as i64],
            resized_image_array,
        ))?;

        let run_options = RunOptions::new()?;
        let outputs = session
            .run_async(ort::inputs![input_name => input_tensor], &run_options)?
            .await?;
        let predictions = outputs[output_name].try_extract_array::<f32>()?;

        println!("{predictions:?}");

        Ok(Value::Null)
    }
}

impl OrtEngine {
    pub async fn new(
        config: OrtConfig,
        model_dir: PathBuf,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let session = Session::builder()?
            .with_execution_providers([
                #[cfg(feature = "tensorrt")]
                ep::TensorRT::default().build(),
                #[cfg(feature = "cuda")]
                ep::CUDA::default().build(),
                #[cfg(feature = "migraphx")]
                ep::MIGraphX::default().build(),
                #[cfg(feature = "openvino")]
                ep::OpenVINO::default().build(),
                #[cfg(feature = "directml")]
                ep::DirectML::default().build(),
                #[cfg(feature = "coreml")]
                ep::CoreML::default().build(),
                #[cfg(feature = "cpu")]
                ep::CPU::default().build(),
            ])
            .map_err(|e| Box::<dyn Error + Send + Sync>::from(e.to_string()))?
            .commit_from_file(model_dir.join("model.onnx"))?;

        Ok(Self {
            config,
            session: Mutex::new(session),
        })
    }
}
