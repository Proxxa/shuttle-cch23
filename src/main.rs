#![recursion_limit = "512"]

use std::sync::Mutex;

use reqwest::Client;
use rocket::{
    fs::{FileServer, Options},
    get, routes,
};
use sqlx::{Executor as _, PgPool};

mod day_eight;
mod day_eleven;
mod day_four;
mod day_fourteen;
mod day_one;
mod day_seven;
mod day_six;
mod day_thirteen;
mod day_twelve;
mod example_day;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

pub struct HuntPool(pub(crate) PgPool);

#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_rocket::ShuttleRocket {
    pool.execute(include_str!("../schema.sql"))
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    let rocket = rocket::build()
        .manage(HuntPool(pool))
        .mount("/", routes![index])
        .mount("/-1", routes![example_day::error])
        .mount(
            "/1",
            routes![/*day_one::bit_cube, */ day_one::bit_sled_cube],
        )
        .mount("/4", routes![day_four::strength, day_four::contest])
        .mount("/6", routes![day_six::endpoint])
        .mount(
            "/7",
            routes![day_seven::b64_decode, day_seven::bake_cookies],
        )
        .mount("/8", routes![day_eight::weight, day_eight::drop])
        .manage(Client::builder().build().unwrap())
        .mount("/11", routes![day_eleven::bull_mode])
        .mount("/11/assets", FileServer::new("assets", Options::None))
        .mount(
            "/12",
            routes![
                day_twelve::save,
                day_twelve::load,
                day_twelve::ulids,
                day_twelve::ulids_weekday
            ],
        )
        .manage(day_twelve::TimedStrings(Default::default()))
        .mount(
            "/13",
            routes![
                day_thirteen::sql,
                day_thirteen::reset,
                day_thirteen::post_orders,
                day_thirteen::orders_total,
                day_thirteen::orders_popular
            ],
        )
        .mount(
            "/14",
            routes![day_fourteen::unsafe_html, day_fourteen::safe_html],
        );

    Ok(rocket.into())
}
