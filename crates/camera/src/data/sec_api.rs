use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Post request

#[derive(Serialize)]
pub struct SecPostRequestHeader {
    pub nonce: String,
    pub opaque: String,
    pub realm: String,
    pub response: String,
    pub user: String,
}

#[derive(Serialize)]
pub struct SecPostRequest {
    pub header: SecPostRequestHeader,
    pub data: Option<Value>,
}

/// Post response

#[derive(Deserialize)]
pub struct SecPostResponseHeader {
    pub status: u8,
    pub message: String,
}

#[derive(Deserialize)]
pub struct SecPostResponse {
    pub header: SecPostResponseHeader,
    pub data: Option<Value>,
}

/// Challenge request

#[derive(Serialize)]
pub struct SecChallengeRequestData {
    pub user: String,
}

#[derive(Serialize)]
pub struct SecChallengeRequest {
    pub data: SecChallengeRequestData,
}

/// Challenge response

#[derive(Deserialize, Clone)]
pub struct SecChallengeResponseData {
    pub realm: String,
    pub nonce: String,
    pub opaque: String,
    pub salt: Vec<u8>,
}

#[derive(Deserialize)]
pub struct SecChallengeResponse {
    pub header: SecPostResponseHeader,
    pub challenge: SecChallengeResponseData,
}
