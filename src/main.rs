#![recursion_limit = "512"]

use day_19::{RoomSenderHolder, TweetViewCounter};
use reqwest::Client;
use rocket::{
    fs::{FileServer, Options},
    get, routes,
};
use shuttle_secrets::SecretStore;
use sqlx::{Executor as _, PgPool};

mod day_x;
mod day_1;
mod day_4;
mod day_5;
mod day_6;
mod day_7;
mod day_8;
mod day_11;
mod day_12;
mod day_13;
mod day_14;
mod day_15;
mod day_18;
mod day_19;
mod day_20;
mod day_21;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

pub struct HuntPool(pub(crate) PgPool);

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] pool: PgPool,
    #[shuttle_secrets::Secrets] secrets: SecretStore,
) -> shuttle_rocket::ShuttleRocket {
    pool.execute(include_str!("../schema.sql"))
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    let rocket = rocket::build()
        .manage(HuntPool(pool))
        .mount("/", routes![index])
        .mount("/-1", routes![day_x::error])
        .mount("/1", routes![/*day_one::bit_cube, */ day_1::bit_sled_cube])
        .mount("/4", routes![day_4::strength, day_4::contest])
        .mount("/5", routes![day_5::slicing, day_5::splitting])
        .mount("/6", routes![day_6::endpoint])
        .mount("/7", routes![day_7::b64_decode, day_7::bake_cookies])
        .mount("/8", routes![day_8::weight, day_8::drop])
        .manage(Client::builder().build().unwrap())
        .mount("/11", routes![day_11::bull_mode])
        .mount("/11/assets", FileServer::new("assets", Options::None))
        .mount(
            "/12",
            routes![
                day_12::save,
                day_12::load,
                day_12::ulids,
                day_12::ulids_weekday
            ],
        )
        .manage(day_12::TimedStrings::default())
        .mount(
            "/13",
            routes![
                day_13::sql,
                day_13::reset,
                day_13::post_orders,
                day_13::orders_total,
                day_13::orders_popular
            ],
        )
        .mount("/14", routes![day_14::unsafe_html, day_14::safe_html])
        .mount("/15", routes![day_15::nice, day_15::game])
        .mount(
            "/18",
            routes![
                day_13::reset,
                day_13::post_orders,
                day_18::post_regions,
                day_18::regions_total,
                day_18::regions_top
            ],
        )
        .manage(TweetViewCounter::default())
        .manage(RoomSenderHolder::default())
        .mount(
            "/19",
            routes![
                day_19::ping_pong,
                day_19::twitter_sock,
                day_19::reset_views,
                day_19::get_views
            ],
        )
        .mount(
            "/20",
            routes![
                day_20::archive_files,
                day_20::archive_files_size,
                day_20::cookie
            ],
        )
        .manage(secrets)
        .mount("/21", routes![day_21::coords, day_21::country]);

    Ok(rocket.into())
}
