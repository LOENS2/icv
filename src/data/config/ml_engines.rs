use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrtExecutionProvider {
    Cpu,
    DirectML,
    CoreML,
    OpenVINO,
    MIGraphX,
    CUDA,
}
#[derive(Deserialize)]
pub struct OrtConfig {
    execution_provider: OrtExecutionProvider,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MlEngineBackend {
    Ort {
        #[serde(rename = "ort")]
        ort_config: OrtConfig,
    },
    HailoRT,
}

#[derive(Deserialize)]
pub struct MlEngineConfig {
    backend: MlEngineBackend,
    model_dir: PathBuf,
}
