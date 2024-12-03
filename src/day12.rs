use std::{collections::HashMap, time::Instant};

use actix_web::{get, post, web, HttpResponse, Responder};
use chrono::{DateTime, Datelike, Utc};
use tokio::sync::Mutex;
use ulid::{serde::ulid_as_uuid, Ulid};

use crate::ShuttleResult;

lazy_static::lazy_static!(
    static ref STATE: web::Data<State> = web::Data::default();
);

#[derive(Default)]
struct State {
    map: Mutex<HashMap<String, Instant>>,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(save)
        .service(load)
        .service(ulids)
        .service(ulids_weekday)
        .app_data(STATE.clone());
}

#[post("/12/save/{s}")]
async fn save(path: web::Path<String>, state: web::Data<State>) -> impl Responder {
    let mut data = state.map.lock().await;

    data.insert(path.into_inner(), Instant::now());

    HttpResponse::Ok()
}

#[get("/12/load/{s}")]
async fn load(path: web::Path<String>, state: web::Data<State>) -> impl Responder {
    let data = state.map.lock().await;

    data[&path.into_inner()].elapsed().as_secs().to_string()
}

#[derive(serde::Serialize, serde::Deserialize)]
struct UlidUuid(#[serde(serialize_with = "ulid_as_uuid::serialize")] Ulid);

#[post("/12/ulids")]
async fn ulids(ulids_json: web::Json<Vec<UlidUuid>>) -> ShuttleResult<impl Responder> {
    let mut ulids = ulids_json.into_inner();
    ulids.reverse();

    Ok(HttpResponse::Ok().json(ulids))
}

#[post("/12/ulids/{weekday}")]
async fn ulids_weekday(
    path: web::Path<u32>,
    ulids_json: web::Json<Vec<Ulid>>,
) -> ShuttleResult<impl Responder> {
    let path = path.into_inner();

    let mut christmas = 0;
    let mut weekday = 0;
    let mut future = 0;
    let mut lsb = 0;

    for ulid in ulids_json.into_inner() {
        let time: DateTime<Utc> = ulid.datetime().into();

        if time.month() == 12 && time.day() == 24 {
            christmas += 1;
        }

        if time.weekday().num_days_from_monday() == path {
            weekday += 1;
        }

        if time > Utc::now() {
            future += 1;
        }

        if ulid.random() & 1 == 1 {
            lsb += 1;
        }
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "christmas eve": christmas,
        "weekday": weekday,
        "in the future": future,
        "LSB is 1": lsb,
    })))
}
