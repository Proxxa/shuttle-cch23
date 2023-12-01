use rocket::{get, routes};

mod day_one;
mod example_day;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[shuttle_runtime::main]
async fn main() -> shuttle_rocket::ShuttleRocket {
    let rocket = rocket::build()
        .mount("/", routes![index])
        .mount("/-1", routes![example_day::error])
        .mount(
            "/1",
            routes![/*day_one::bit_cube, */ day_one::bit_sled_cube],
        );

    Ok(rocket.into())
}
