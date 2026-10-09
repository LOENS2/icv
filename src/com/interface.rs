use crate::data::com::{ComRequest, ComResponse};
use async_trait::async_trait;
use std::error::Error;
use tokio::sync::mpsc::{Receiver, Sender};

#[async_trait]
pub trait ComInterface {
    async fn receive_data(&mut self, inbound_tx: Sender<ComRequest>) -> Result<(), Box<dyn Error>>;

    async fn send_data(
        &mut self,
        outbound_rx: &mut Receiver<ComResponse>,
    ) -> Result<(), Box<dyn Error>>;
}
