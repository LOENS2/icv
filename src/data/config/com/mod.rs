use crate::data::config::com::mqtt::MqttConfig;
use crate::data::config::com::opcua::OpcUaConfig;
use serde::Deserialize;

pub mod mqtt;
pub mod opcua;

#[derive(Deserialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum ComConfig {
    Opcua {
        #[serde(rename = "opcua")]
        opcua_config: OpcUaConfig,
    },
    Mqtt {
        #[serde(rename = "mqtt")]
        mqtt_config: MqttConfig,
    },
}
