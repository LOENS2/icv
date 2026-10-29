pub mod ort;

use crate::data::config::ml_engines::ort::OrtConfig;
use getset::{CloneGetters, Getters};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum MlEngineBackend {
    Ort {
        #[serde(rename = "ort")]
        ort_config: OrtConfig,
    },
    HailoRT,
}

#[derive(Deserialize, Getters, CloneGetters, Clone)]
pub struct MlEngineConfig {
    #[getset(get_clone = "pub")]
    backend: MlEngineBackend,
    #[getset(get = "pub")]
    model_dir: PathBuf,
}
