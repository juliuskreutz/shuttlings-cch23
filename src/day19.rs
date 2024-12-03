use actix_web::{
    get,
    web::{self, ServiceConfig},
    HttpRequest, Responder,
};
use actix_ws::Message;

use crate::ShuttleResult;

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(ws_ping);
}
#[get("/19/ws/ping")]
async fn ws_ping(req: HttpRequest, body: web::Payload) -> ShuttleResult<impl Responder> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    actix_web::rt::spawn(async move {
        let mut started = false;

        while let Some(Ok(msg)) = msg_stream.recv().await {
            if let Message::Text(msg) = msg {
                match msg.to_string().as_str() {
                    "serve" => started = true,
                    "ping" if started => session.text("pong").await.unwrap(),
                    _ => {}
                }
            }
        }

        let _ = session.close(None).await;
    });

    Ok(response)
}
