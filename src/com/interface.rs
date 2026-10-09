use crate::data::com::{ComRequest, ComResponse};
use async_trait::async_trait;
use std::error::Error;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;

#[async_trait]
pub trait ComInterface {
    async fn start_receive(
        &mut self,
        inbound_tx: Sender<ComRequest>,
    ) -> Result<JoinHandle<Result<(), Box<dyn Error + Send + Sync>>>, Box<dyn Error + Send + Sync>>;

    async fn start_send(
        &mut self,
        outbound_rx: Receiver<ComResponse>,
    ) -> Result<JoinHandle<Result<(), Box<dyn Error + Send + Sync>>>, Box<dyn Error + Send + Sync>>;
}
