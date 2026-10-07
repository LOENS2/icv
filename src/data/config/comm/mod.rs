use crate::data::config::comm::mqtt::MqttConfig;
use crate::data::config::comm::opcua::OpcUaConfig;
use serde::Deserialize;

pub mod mqtt;
pub mod opcua;

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
