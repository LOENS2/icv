#[cfg(test)]
mod tests {
    use crate::com::interface::ComInterface;
    use crate::com::mqtt::ComMqtt;
    use crate::data::com::{ComRequest, ComResponse};
    use crate::data::config::com::mqtt::MqttConfig;
    use log::debug;
    use serde_json::{Value, json};
    use simple_logger::SimpleLogger;
    use std::env;
    use std::error::Error;
    use std::time::Duration;
    use tokio::sync::mpsc;
    use tokio::time;

    const BUFFER_SIZE: usize = 100;

    fn get_mqtt_config() -> Result<MqttConfig, Box<dyn Error + Send + Sync>> {
        let mqtt_config_json = json!({
            "host": env::var("MQTT_HOST")?,
            "port": env::var("MQTT_PORT")?.parse::<u16>()?,
            "client_id": "my_test_client",
            "username": env::var("MQTT_USER")?,
            "password": env::var("MQTT_PASS")?,
            "topic": "test/mqtttopic",
            "tls": false,
        });

        let mqtt_config: MqttConfig = serde_json::from_value(mqtt_config_json)?;

        Ok(mqtt_config)
    }

    #[tokio::test]
    async fn test_connection_plain() -> Result<(), Box<dyn Error + Send + Sync>> {
        let mqtt_config = get_mqtt_config()?;

        let _mqtt = ComMqtt::new(mqtt_config).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_send() -> Result<(), Box<dyn Error + Send + Sync>> {
        SimpleLogger::new().init()?;

        let mqtt_config = get_mqtt_config()?;

        let mut mqtt = ComMqtt::new(mqtt_config).await?;

        let (inbound_tx, mut inbound_rx) = mpsc::channel::<ComRequest>(BUFFER_SIZE);
        let (outbound_tx, outbound_rx) = mpsc::channel::<ComResponse>(BUFFER_SIZE);

        mqtt.start_receive(inbound_tx).await?;
        mqtt.start_send(outbound_rx).await?;

        tokio::spawn(async move {
            while let Some(_req) = inbound_rx.recv().await {
                println!("Inbound request received");
            }

            debug!("Inbound RX loop has exited. This should not have happened!")
        });

        time::sleep(Duration::from_secs(1)).await;

        for _ in 0..10 {
            let request = ComResponse {
                request_id: "same_id".to_string(),
                result: vec![Value::Null],
            };

            outbound_tx.send(request).await?;

            time::sleep(Duration::from_millis(50)).await;
        }

        Ok(())
    }
}
