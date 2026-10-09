use getset::Getters;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Getters)]
#[getset(get_copy = "pub")]
pub struct ComRequest {
    request_id: String,
    batches: u16,
}

#[derive(Serialize)]
pub struct ComResponse {
    pub request_id: String,
    pub result: Vec<Value>,
}
