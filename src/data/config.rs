use rust_mqtt::types::QoS;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct OpcUaConfig {
    host: String,
    port: u16,
    username: Option<String>,
    password: Option<String>,
    keep_alive: u16,
}

#[derive(Deserialize)]
pub enum MqttQoS {
    eio = 0,
    mqtt = 1,
}

#[derive(Deserialize)]
pub struct MqttConfig {
    host: String,
    port: u16,
    client_id: String,
    qos: Option<MqttQoS>,
    topic: String,
    tls: Option<bool>,
    cert_path: Option<String>,
    key_path: Option<String>,
}

#[derive(Deserialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum CommConfig {
    Opcua { opcua_config: OpcUaConfig },
    Mqtt { mqtt_config: MqttConfig },
}

struct IcvConfig {}
