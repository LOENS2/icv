pub mod ort;

use crate::data::config::ml_engines::ort::OrtConfig;
use serde::Deserialize;
use std::path::PathBuf;

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
