use std::collections::HashMap;

use ::serde::{Deserialize, Serialize};
use rocket::{response::status::BadRequest, serde::json::Json, *};
use serde_json::Value;
use sqlx::{Executor as _, FromRow};

use crate::HuntPool;

/// Helper type for deserializable types that aren't FromRow
#[derive(Deserialize, FromRow)]
struct RowType<T>(pub T);

#[get("/sql")]
pub async fn sql(db: &State<HuntPool>) -> Result<Json<i32>, BadRequest<String>> {
    sqlx::query_scalar("SELECT 20231213")
        .fetch_one(&db.0)
        .await
        .map_err(|e| BadRequest(e.to_string()))
        .map(|i| Json(i))
}

#[post("/reset")]
pub async fn reset(db: &State<HuntPool>) -> Result<(), BadRequest<String>> {
    db.0.execute(include_str!("../schema.sql"))
        .await
        .map(|_| ())
        .map_err(|e| BadRequest(e.to_string()))
}

#[derive(Serialize, Deserialize, FromRow, Debug)]
#[serde(crate = "rocket::serde")]
pub struct Order {
    id: i32,
    region_id: i32,
    gift_name: String,
    quantity: i32,
}

#[post("/orders", data = "<data>")]
pub async fn post_orders(
    db: &State<HuntPool>,
    data: Json<Vec<Order>>,
) -> Result<(), BadRequest<String>> {
    for order in dbg!(data).iter() {
        sqlx::query(
            "INSERT INTO orders (id, region_id, gift_name, quantity) VALUES ($1, $2, $3, $4)",
        )
        .bind(order.id)
        .bind(order.region_id)
        .bind(order.gift_name.clone())
        .bind(order.quantity)
        .execute(&db.0)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;
    }

    Ok(())
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct TotalOrders {
    total: i64,
}

#[get("/orders/total")]
pub async fn orders_total(db: &State<HuntPool>) -> Result<Json<TotalOrders>, BadRequest<String>> {
    sqlx::query_as("SELECT SUM(quantity) FROM orders")
        .fetch_one(&db.0)
        .await
        .map(|i: RowType<i64>| Json(TotalOrders { total: i.0 }))
        .map_err(|e| BadRequest(e.to_string()))
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct PopularOrder {
    popular: Value,
}

#[get("/orders/popular")]
pub async fn orders_popular(
    db: &State<HuntPool>,
) -> Result<Json<PopularOrder>, BadRequest<String>> {
    sqlx::query_as("SELECT * FROM orders")
        .fetch_all(&db.0)
        .await
        .map_err(|e| BadRequest(e.to_string()))
        .map(|a: Vec<Order>| {
            dbg!(a)
                .iter()
                .fold(HashMap::<String, i32>::new(), |mut hm, rs| {
                    // This clone is costly, but I don't know how to avoid it.
                    hm.entry(rs.gift_name.clone())
                        .and_modify(|i| *i += rs.quantity)
                        .or_default();
                    hm
                })
                .iter()
                .max_by(|x, y| x.1.cmp(y.1))
                .map(|(s, _)| s.to_owned())
        })
        .map(|opt| {
            Json(PopularOrder {
                popular: match opt {
                    Some(s) => Value::String(s),
                    _ => Value::Null,
                },
            })
        })
}
