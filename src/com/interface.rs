use crate::data::com::{ComRequest, ComResponse};
use async_trait::async_trait;
use std::error::Error;
use tokio::sync::mpsc::{Receiver, Sender};

#[async_trait]
pub trait ComInterface {
    async fn start_receive(
        &mut self,
        inbound_tx: Sender<ComRequest>,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;

    async fn start_send(
        &mut self,
        outbound_rx: Receiver<ComResponse>,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
}
