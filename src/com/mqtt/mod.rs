use crate::com::interface::ComInterface;
use crate::data::com::{ComRequest, ComResponse};
use crate::data::config::com::mqtt::{MqttConfig, MqttQoS};
use async_trait::async_trait;
use image::EncodableLayout;
use rumqttc::v5::AsyncClient;
use rumqttc::v5::Incoming;
use rumqttc::v5::MqttOptions;
use rumqttc::v5::mqttbytes::QoS;
use rumqttc::v5::{Event, EventLoop};
use serde_json::ser::to_string;
use std::error::Error;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;

pub struct ComMqtt<'config> {
    config: &'config MqttConfig,
    client: AsyncClient,
    event_loop: Option<EventLoop>,
    qos: QoS,
}

#[async_trait]
impl<'config> ComInterface for ComMqtt<'config> {
    async fn start_receive(
        &mut self,
        inbound_tx: Sender<ComRequest>,
    ) -> Result<JoinHandle<Result<(), Box<dyn Error + Send + Sync>>>, Box<dyn Error + Send + Sync>>
    {
        let mut event_loop = match self.event_loop.take() {
            Some(event_loop) => event_loop,
            None => {
                return Err(
                    "Event loop has already been taken. You likely called start_receive twice!"
                        .into(),
                );
            }
        };

        self.client.subscribe(self.config.topic(), self.qos).await?;

        let topic = self.config.topic().to_owned();
        let handle = tokio::spawn(async move {
            while let Event::Incoming(Incoming::Publish(publish)) = event_loop.poll().await? {
                if publish.topic != topic {
                    continue;
                }

                let payload_data: ComRequest = serde_json::from_slice(publish.payload.as_bytes())?;

                inbound_tx.send(payload_data).await?;
            }

            Ok(())
        });

        Ok(handle)
    }

    async fn start_send(
        &mut self,
        mut outbound_rx: Receiver<ComResponse>,
    ) -> Result<JoinHandle<Result<(), Box<dyn Error + Send + Sync>>>, Box<dyn Error + Send + Sync>>
    {
        let qos = self.qos.clone();
        let topic = self.config.topic().to_owned();
        let client = self.client.clone();

        let handle = tokio::spawn(async move {
            while let Some(data) = outbound_rx.recv().await {
                let payload = to_string(&data)?;

                client.publish(&topic, qos, false, payload).await?;
            }

            Ok(())
        });

        Ok(handle)
    }
}

impl<'config> ComMqtt<'config> {
    const CHANNEL_CAPACITY: usize = 10;
    pub async fn new(config: &'config MqttConfig) -> Result<Self, Box<dyn Error>> {
        let mut mqtt_options = MqttOptions::new(config.client_id(), config.host(), config.port());

        if let Some(username) = config.username()
            && let Some(password) = config.password()
        {
            mqtt_options.set_credentials(username, password);
        }

        if let Some(tls) = config.tls()
            && tls
        {
            todo!("TLS not implemented yet")
        }

        let (client, event_loop) = AsyncClient::new(mqtt_options, Self::CHANNEL_CAPACITY);

        let qos = match config.qos() {
            Some(qos) => match qos {
                MqttQoS::AtMostOnce => QoS::AtMostOnce,
                MqttQoS::AtLeastOnce => QoS::AtLeastOnce,
                MqttQoS::ExactlyOnce => QoS::ExactlyOnce,
            },
            None => QoS::AtMostOnce,
        };

        Ok(Self {
            config,
            client,
            event_loop: Some(event_loop),
            qos,
        })
    }
}
