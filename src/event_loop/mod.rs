use crate::camera::interface::CameraInterface;
use crate::com::interface::ComInterface;
use crate::data::com::{ComRequest, ComResponse};
use crate::ml_engines::interface::MlEngineInterface;
use opcua::core::tracing::error;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

pub async fn run_app(
    camera: Arc<dyn CameraInterface>,
    ml_engine: Arc<dyn MlEngineInterface>,
    mut communication: Box<dyn ComInterface>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    const BUFFER_SIZE: usize = 100;

    let (inbound_tx, mut inbound_rx) = mpsc::channel::<ComRequest>(BUFFER_SIZE);
    let (outbound_tx, outbound_rx) = mpsc::channel::<ComResponse>(BUFFER_SIZE);

    communication.start_receive(inbound_tx).await?.await??;
    communication.start_send(outbound_rx).await?.await??;

    while let Some(msg) = inbound_rx.recv().await {
        let camera = camera.clone();
        let ml_engine = ml_engine.clone();
        let outbound_tx = outbound_tx.clone();

        tokio::spawn(async move {
            if let Err(e) = run_pipeline(msg, outbound_tx, camera, ml_engine).await {
                error!("Pipeline error occurred: {e}");
            }
        });
    }

    Ok(())
}

async fn run_pipeline(
    msg: ComRequest,
    outbound_tx: Sender<ComResponse>,
    camera: Arc<dyn CameraInterface>,
    ml_engine: Arc<dyn MlEngineInterface>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let image = camera.capture_image().await?;

    let data = ml_engine.predict(image).await?;

    let response = ComResponse {
        request_id: msg.request_id().to_owned(),
        result: vec![data],
    };

    outbound_tx.send(response).await?;

    Ok(())
}
