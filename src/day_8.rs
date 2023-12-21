use std::collections::HashMap;

use reqwest::Client;
use rocket::{http::Status, serde::json::Json, *};
use serde_json::Value;

const ENDPOINT_BASE: &str = "https://pokeapi.co/api/v2/pokemon/";
#[get("/weight/<id>")]
pub async fn weight(id: usize, client: &State<Client>) -> Result<Json<f64>, Status> {
    client
        .get(format!("{ENDPOINT_BASE}{id}"))
        .send()
        .await
        .map_err(|e| Status {
            code: e.status().map_or(500, |s| s.as_u16()),
        })?
        .json::<HashMap<String, Value>>()
        .await
        .map_err(|_| Status { code: 500 })?
        .get("weight")
        .and_then(|v| v.as_f64())
        .map(|f| Json(f / 10f64))
        .ok_or(Status { code: 500 })
}

// const GRAVITY: f64 = 9.825;
// v_f^2 - v_i^2 = 2 * dx * a
// v_i = 0
// sqrt(2 * 10 * 9.825) = sqrt(196.5) = ~14.0178457689
// Thanks, calculator!
const VELOCITY_AFTER_10M: f64 = 14.0178457689;

#[get("/drop/<id>")]
pub async fn drop(id: usize, client: &State<Client>) -> Result<Json<f64>, Status> {
    weight(id, client)
        .await
        .map(|Json(f)| Json(f * VELOCITY_AFTER_10M))
}
