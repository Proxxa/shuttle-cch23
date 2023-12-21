use ::serde::{Deserialize, Serialize};
use rocket::{http::Status, serde::json::Json, *};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct StrengthData {
    #[allow(unused)]
    name: String,
    strength: usize,
}
#[post("/strength", data = "<data>")]
pub fn strength(data: Json<Vec<StrengthData>>) -> Json<usize> {
    Json(data.iter().fold(0, |a, b| a + b.strength))
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ContestData {
    name: String,
    strength: usize,
    speed: f32,
    height: usize,
    antler_width: usize,
    snow_magic_power: usize,
    favorite_food: String,
    #[serde(alias = "cAnD13s_3ATeN-yesT3rdAy")]
    candies_eaten_yesterday: usize,
}

#[derive(Serialize, Default)]
#[serde(crate = "rocket::serde")]
pub struct ContestOutput {
    fastest: String,
    tallest: String,
    magician: String,
    consumer: String,
}

#[post("/contest", data = "<data>")]
pub fn contest(data: Json<Vec<ContestData>>) -> Result<Json<ContestOutput>, Status> {
    let fastest = data
        .iter()
        .max_by(|a, b| a.speed.total_cmp(&b.speed))
        .ok_or(Status { code: 400 })?;
    let fastest = format!(
        "Speeding past the finish line with a strength of {} is {}",
        fastest.strength, &fastest.name
    );

    let tallest = data
        .iter()
        .max_by_key(|r| r.height)
        .ok_or(Status { code: 400 })?;
    let tallest = format!(
        "{} is standing tall with his {} cm wide antlers",
        &tallest.name, tallest.antler_width
    );

    let magician = data
        .iter()
        .max_by_key(|r| r.snow_magic_power)
        .ok_or(Status { code: 400 })?;
    let magician = format!(
        "{} could blast you away with a snow magic power of {}",
        &magician.name, magician.snow_magic_power
    );

    let consumer = data
        .iter()
        .max_by_key(|r| r.candies_eaten_yesterday)
        .ok_or(Status { code: 400 })?;
    let consumer = format!(
        "{} ate lots of candies, but also some {}",
        &consumer.name, &consumer.favorite_food
    );

    Ok(Json(ContestOutput {
        fastest,
        tallest,
        magician,
        consumer,
    }))
}
