use crate::com::interface::ComInterface;
use crate::data::com::{ComRequest, ComResponse};
use crate::data::config::com::mqtt::{MqttConfig, MqttQoS};
use async_trait::async_trait;
use image::EncodableLayout;
use log::{debug, error};
use rumqttc::v5::AsyncClient;
use rumqttc::v5::Incoming;
use rumqttc::v5::MqttOptions;
use rumqttc::v5::mqttbytes::QoS;
use rumqttc::v5::{Event, EventLoop};
use serde_json::ser::to_string;
use std::error::Error;
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;

pub struct ComMqtt {
    config: MqttConfig,
    client: AsyncClient,
    event_loop: Option<EventLoop>,
    qos: QoS,
}

#[async_trait]
impl ComInterface for ComMqtt {
    async fn start_receive(
        &mut self,
        inbound_tx: Sender<ComRequest>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let event_loop = match self.event_loop.take() {
            Some(event_loop) => event_loop,
            None => {
                return Err(
                    "Event loop has already been taken. You likely called start_receive twice!"
                        .into(),
                );
            }
        };

        let topic = format!("{}/command", self.config.topic());

        self.client.subscribe(&topic, self.qos).await?;

        tokio::spawn(async move {
            if let Err(e) = Self::receive_task(event_loop, topic, inbound_tx).await {
                error!("Error in start receive: {e:?}");
            }
        });

        Ok(())
    }

    async fn start_send(
        &mut self,
        outbound_rx: Receiver<ComResponse>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let qos = self.qos.clone();
        let topic = format!("{}/result", self.config.topic());
        let client = self.client.clone();

        tokio::spawn(async move {
            if let Err(e) = Self::send_task(outbound_rx, client, topic, qos).await {
                error!("Error in start send: {e:?}");
            }
        });

        Ok(())
    }
}

impl ComMqtt {
    const CHANNEL_CAPACITY: usize = 10;
    pub async fn new(config: MqttConfig) -> Result<Self, Box<dyn Error + Send + Sync>> {
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

    async fn receive_task(
        mut event_loop: EventLoop,
        topic: String,
        inbound_tx: Sender<ComRequest>,
    ) -> Result<(), Box<dyn Error>> {
        loop {
            let event = event_loop.poll().await?;

            match event {
                Event::Incoming(Incoming::Publish(publish)) => {
                    if publish.topic != topic {
                        continue;
                    }

                    let payload_data: ComRequest =
                        serde_json::from_slice(publish.payload.as_bytes())?;

                    inbound_tx.send(payload_data).await?;
                }
                _ => {}
            }
        }
    }

    async fn send_task(
        mut outbound_rx: Receiver<ComResponse>,
        client: AsyncClient,
        topic: String,
        qos: QoS,
    ) -> Result<(), Box<dyn Error>> {
        while let Some(data) = outbound_rx.recv().await {
            let payload = to_string(&data)?;

            client.publish(&topic, qos, false, payload).await?;
        }

        debug!("Send loop has exited. This is bad!");

        Ok::<(), Box<dyn Error>>(())
    }
}
