use serde::Deserialize;
use serde_repr::Deserialize_repr;

#[derive(Deserialize)]
pub struct OpcUaConfig {
    host: String,
    port: u16,
    username: Option<String>,
    password: Option<String>,
}

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

#[derive(Deserialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum CommConfig {
    Opcua {
        #[serde(rename = "opcua")]
        opcua_config: OpcUaConfig,
    },
    Mqtt {
        #[serde(rename = "mqtt")]
        mqtt_config: MqttConfig,
    },
}

#[derive(Deserialize)]
pub struct SickSecConfig {
    host: String,
    username: String,
    password: String,
}

#[derive(Deserialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum CameraConfig {
    SickSec {
        #[serde(rename = "sick_sec")]
        sick_sec_config: SickSecConfig,
    },
}

#[derive(Deserialize)]
pub struct IcvConfig {
    communication: CommConfig,
    camera: CameraConfig,
}
