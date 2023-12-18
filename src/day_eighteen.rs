use ::serde::{Deserialize, Serialize};
use rocket::{http::Status, response::status::BadRequest, serde::json::Json, *};

use crate::HuntPool;

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct Region {
    pub id: i32,
    pub name: String,
}

#[post("/regions", data = "<data>")]
pub async fn post_regions(
    db: &State<HuntPool>,
    data: Json<Vec<Region>>,
) -> Result<(), BadRequest<String>> {
    for region in data.iter() {
        sqlx::query("INSERT INTO regions (id, name) VALUES ($1, $2)")
            .bind(region.id)
            .bind(region.name.clone())
            .execute(&db.0)
            .await
            .map_err(|e| BadRequest(e.to_string()))?;
    }

    Ok(())
}

#[derive(Serialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct RegionOrders {
    region: String,
    total: i64,
}

struct RegionOrderCollection(Vec<RegionOrders>);
impl FromIterator<(String, i64)> for RegionOrderCollection {
    fn from_iter<T: IntoIterator<Item = (String, i64)>>(iter: T) -> Self {
        Self(iter.into_iter().map(RegionOrders::from).collect())
    }
}

impl From<(String, i64)> for RegionOrders {
    fn from((region, total): (String, i64)) -> Self {
        Self { region, total }
    }
}

#[get("/regions/total")]
pub async fn regions_total(
    db: &State<HuntPool>,
) -> Result<Json<Vec<RegionOrders>>, (Status, String)> {
    Ok(Json(
        sqlx::query_as(
            r#"SELECT regions.name, SUM(orders.quantity) AS total_orders FROM orders
                INNER JOIN regions ON orders.region_id = regions.id
                GROUP BY name
                ORDER BY name ASC;"#,
        )
        .fetch_all(&db.0)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?
        .into_iter()
        .collect::<RegionOrderCollection>()
        .0,
    ))
}

#[derive(Serialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct RegionTopGifts {
    region: String,
    top_gifts: Vec<String>,
}

struct RegionTopGiftCollection(Vec<RegionTopGifts>);

impl FromIterator<(String, Vec<String>)> for RegionTopGiftCollection {
    fn from_iter<T: IntoIterator<Item = (String, Vec<String>)>>(iter: T) -> Self {
        Self(iter.into_iter().map(RegionTopGifts::from).collect())
    }
}

impl From<(String, Vec<String>)> for RegionTopGifts {
    fn from((region, top_gifts): (String, Vec<String>)) -> Self {
        Self { region, top_gifts }
    }
}

#[get("/regions/top_list/<num>")]
pub async fn regions_top(
    db: &State<HuntPool>,
    num: i64,
) -> Result<Json<Vec<RegionTopGifts>>, (Status, String)> {
    let mut vec: Vec<(String, Vec<String>)> = Vec::new();
    for (id, region) in sqlx::query_as::<_, (i32, String)>("SELECT * FROM regions ORDER BY name")
        .fetch_all(&db.0)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?
        .into_iter()
    {
        vec.push((region, sqlx::query_scalar(r#"SELECT gift_name FROM orders WHERE region_id = $1 GROUP BY gift_name ORDER BY SUM(quantity) DESC, gift_name ASC LIMIT $2"#)
            .bind(id)
            .bind(num)
            .fetch_all(&db.0)
            .await
            .map_err(|e| (Status::InternalServerError, e.to_string()))?
            .into_iter()
            .collect::<Vec<String>>()));
    }

    Ok(Json(
        vec.into_iter().collect::<RegionTopGiftCollection>().0
    ))
}
