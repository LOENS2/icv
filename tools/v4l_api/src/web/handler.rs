use crate::data::web::AppState;
use actix_web::{HttpResponse, Responder, get, web};
use tokio_util::io::{ReaderStream, SyncIoBridge};

#[get("/get-image")]
pub async fn get_image(data: web::Data<AppState>) -> actix_web::Result<impl Responder> {
    let permit = data
        .capture_gate
        .clone()
        .acquire_owned()
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("capture gate closed: {e}"))
        })?;

    let frame =
        data.v4l_capture.capture_frame().await.map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("capture failed: {e}"))
        })?;

    let (writer, reader) = tokio::io::duplex(64 * 1024);

    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        let bridge = SyncIoBridge::new(writer);
        if let Err(e) = frame.write_png(bridge) {
            log::error!("PNG streaming failed: {e}");
        }
    });

    let stream = ReaderStream::new(reader);
    Ok(HttpResponse::Ok()
        .content_type("image/png")
        .streaming(stream))
}

#[get("/get-frame-nv12")]
pub async fn get_frame_nv12(data: web::Data<AppState>) -> actix_web::Result<impl Responder> {
    let _permit = data
        .capture_gate
        .clone()
        .acquire_owned()
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("capture gate closed: {e}"))
        })?;

    let frame =
        data.v4l_capture.capture_frame().await.map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("capture failed: {e}"))
        })?;

    let width = frame.width();
    let height = frame.height();
    let body = frame.into_tightly_packed_nv12();

    Ok(HttpResponse::Ok()
        .content_type("application/octet-stream")
        .insert_header(("X-Frame-Width", width.to_string()))
        .insert_header(("X-Frame-Height", height.to_string()))
        .insert_header(("X-Frame-Format", "NV12"))
        .body(body))
}
