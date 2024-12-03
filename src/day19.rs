use std::collections::HashMap;

use actix_web::{
    get, post,
    web::{self, ServiceConfig},
    HttpRequest, HttpResponse, Responder,
};
use actix_ws::{Message, Session};
use tokio::sync::Mutex;

use crate::ShuttleResult;

lazy_static::lazy_static!(
    static ref STATE: web::Data<State> = web::Data::default();
);

#[derive(Default)]
struct State {
    counter: Mutex<usize>,
    rooms: Mutex<HashMap<usize, HashMap<String, Session>>>,
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(ws_ping)
        .service(reset)
        .service(views)
        .service(ws_room)
        .app_data(STATE.clone());
}

#[get("/19/ws/ping")]
async fn ws_ping(req: HttpRequest, body: web::Payload) -> ShuttleResult<impl Responder> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    actix_web::rt::spawn(async move {
        let mut started = false;

        while let Some(Ok(msg)) = msg_stream.recv().await {
            if let Message::Text(msg) = msg {
                match msg.as_ref() {
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

#[post("/19/reset")]
async fn reset(state: web::Data<State>) -> impl Responder {
    *state.counter.lock().await = 0;

    HttpResponse::Ok()
}

#[get("/19/views")]
async fn views(state: web::Data<State>) -> impl Responder {
    state.counter.lock().await.to_string()
}

#[get("/19/ws/room/{number}/user/{string}")]
async fn ws_room(
    req: HttpRequest,
    body: web::Payload,
    path: web::Path<(usize, String)>,
    state: web::Data<State>,
) -> ShuttleResult<impl Responder> {
    let (room, user) = path.into_inner();

    let (response, session, mut msg_stream) = actix_ws::handle(&req, body)?;
    state
        .rooms
        .lock()
        .await
        .entry(room)
        .or_default()
        .insert(user.clone(), session.clone());

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.recv().await {
            match msg {
                Message::Text(msg) => {
                    let user = user.clone();

                    let json: serde_json::Value = serde_json::from_str(msg.as_ref()).unwrap();

                    let message = json["message"].as_str().unwrap().to_string();

                    if message.len() > 128 {
                        continue;
                    }

                    for session in state
                        .rooms
                        .lock()
                        .await
                        .get_mut(&room)
                        .unwrap()
                        .values_mut()
                    {
                        session
                            .text(
                                serde_json::to_string(
                                    &serde_json::json!({"user": user, "message": message}),
                                )
                                .unwrap(),
                            )
                            .await
                            .unwrap();

                        *state.counter.lock().await += 1;
                    }
                }
                Message::Close(_) => {
                    state
                        .rooms
                        .lock()
                        .await
                        .get_mut(&room)
                        .unwrap()
                        .remove(&user);

                    let _ = session.close(None).await;

                    break;
                }
                _ => {}
            }
        }
    });

    Ok(response)
}
