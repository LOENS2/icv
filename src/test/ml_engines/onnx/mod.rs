#[cfg(test)]
mod tests {
    use crate::data::config::ml_engines::ort::OrtConfig;
    use crate::ml_engines::interface::MlEngineInterface;
    use crate::ml_engines::onnx::OrtEngine;
    use image::{GenericImageView, ImageReader};
    use serde_json::json;
    use std::env;
    use std::error::Error;
    use std::path::PathBuf;

    #[tokio::test]
    async fn run_test_inference() -> Result<(), Box<dyn Error>> {
        let model_path = PathBuf::from(env::var("MODEL_PATH")?);
        let image = ImageReader::open(env::var("IMAGE_PATH")?)?.decode()?;

        let ort_config_json = json!({
           "execution_provider": "migraphx"
        });

        let ort_config: OrtConfig = serde_json::from_value(ort_config_json)?;

        let mut engine = OrtEngine::new(ort_config, model_path).await?;

        let (image_width, image_height) = image.dimensions();
        let cropped_image = image.crop_imm(
            (image_width - image_height) / 2,
            0,
            image_height,
            image_height,
        );

        let inference_result = engine.predict(cropped_image).await?;

        println!("{inference_result:?}");

        Ok(())
    }
}
