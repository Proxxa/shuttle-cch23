use rocket::{get, routes};

mod day_four;
mod day_one;
mod example_day;
mod day_six;

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
        )
        .mount("/4", routes![day_four::strength, day_four::contest])
        .mount("/6", routes![day_six::endpoint]);

    Ok(rocket.into())
}
