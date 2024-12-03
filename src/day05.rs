use actix_web::{
    post,
    web::{self, ServiceConfig},
    HttpResponse, Responder,
};

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(slice);
}

#[derive(serde::Deserialize)]
struct SliceQuery {
    offset: Option<usize>,
    limit: Option<usize>,
    split: Option<usize>,
}

#[post("/5")]
async fn slice(query: web::Query<SliceQuery>, list: web::Json<Vec<String>>) -> impl Responder {
    let offset = query.offset.unwrap_or_default();
    let max_limit = list.len() - offset;
    let limit = query.limit.unwrap_or(max_limit).min(max_limit);

    let slice = &list[offset..offset + limit];

    if let Some(split) = query.split {
        let mut chunks = Vec::new();

        for chunk in slice.chunks(split) {
            chunks.push(chunk);
        }

        HttpResponse::Ok().json(chunks)
    } else {
        HttpResponse::Ok().json(slice)
    }
}
