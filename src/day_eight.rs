use std::collections::HashMap;

use rocket::{http::Status, serde::json::Json, *};
use serde_json::Value;

const ENDPOINT_BASE: &str = "https://pokeapi.co/api/v2/pokemon/";
async fn get_weight(id: usize) -> Result<f64, reqwest::Error> {
    Ok(reqwest::get(format!("{ENDPOINT_BASE}{id}"))
        .await?
        .json::<HashMap<String, Value>>()
        .await?
        .get("weight")
        .and_then(|v| v.as_f64())
        .map(|f| f / 10f64)
        .unwrap())
    // Unwrap because the weight value is guaranteed to exist
}

#[get("/weight/<id>")]
pub async fn weight(id: usize) -> Result<String, Status> {
    get_weight(id)
        .await
        .map_err(|e| Status {
            code: e.status().map_or(0, |s| s.as_u16()),
        })
        .map(|f| f.to_string())
}

// const GRAVITY: f64 = 9.825;
// v_f^2 - v_i^2 = 2 * dx * a
// v_i = 0
// sqrt(2 * 10 * 9.825) = sqrt(196.5) = ~14.0178457689
// Thanks, calculator!
const VELOCITY_AFTER_10M: f64 = 14.0178457689;

#[get("/drop/<id>")]
pub async fn drop(id: usize) -> Result<Json<f64>, Status> {
    get_weight(id)
        .await
        .map_err(|e| Status {
            code: e.status().map_or(0, |s| s.as_u16()),
        })
        .map(|w| Json(VELOCITY_AFTER_10M * w))
}
