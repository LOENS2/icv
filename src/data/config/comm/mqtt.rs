use serde::Deserialize;
use serde_repr::Deserialize_repr;

#[derive(Deserialize_repr)]
#[repr(u8)]
pub enum MqttQoS {
    AtMostOnce = 0,
    AtLeastOnce = 1,
    ExactlyOnce = 2,
}

#[derive(Deserialize)]
pub struct MqttConfig {
    host: String,
    port: u16,
    client_id: String,
    username: Option<String>,
    password: Option<String>,
    qos: Option<u8>,
    topic: String,
    tls: Option<bool>,
    cert_path: Option<String>,
    key_path: Option<String>,
}
