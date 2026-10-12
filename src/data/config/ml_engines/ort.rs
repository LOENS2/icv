use getset::Getters;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum OrtExecutionProvider {
    Cpu,
    DirectML,
    CoreML,
    OpenVINO,
    MIGraphX,
    CUDA,
}
#[derive(Deserialize, Getters, Clone)]
#[getset(get = "pub")]
pub struct OrtConfig {
    execution_provider: OrtExecutionProvider,
}
