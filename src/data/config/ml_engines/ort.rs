use serde::Deserialize;

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
