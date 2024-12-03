mod day00;
mod day01;
mod day04;
mod day05;
mod day06;
mod day07;
mod day08;
mod day11;
mod day12;
mod day13;
mod day14;
mod day15;
mod day18;
mod day19;
mod day20;

use actix_web::web;
use shuttle_actix_web::ShuttleActixWeb;
use sqlx::PgPool;

type ShuttleResult<T> = Result<T, Box<dyn std::error::Error>>;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] pool: PgPool,
) -> ShuttleActixWeb<impl FnOnce(&mut web::ServiceConfig) + Send + Clone + 'static> {
    let config = move |cfg: &mut web::ServiceConfig| {
        cfg.configure(day00::configure)
            .configure(day01::configure)
            .configure(day04::configure)
            .configure(day05::configure)
            .configure(day06::configure)
            .configure(day07::configure)
            .configure(day08::configure)
            .configure(day11::configure)
            .configure(day12::configure)
            .configure(day13::configure)
            .configure(day14::configure)
            .configure(day15::configure)
            .configure(day18::configure)
            .configure(day19::configure)
            .configure(day20::configure)
            .app_data(web::PayloadConfig::new(5 * 1024 * 1024))
            .app_data(web::Data::new(pool.clone()));
    };

    Ok(config.into())
}
