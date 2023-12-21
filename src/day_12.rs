use std::{collections::HashMap, sync::Mutex, time::SystemTime};

use ::serde::Serialize;
use chrono::{DateTime, Datelike, Utc};
use rocket::{http::Status, serde::json::Json, *};
use ulid::Ulid;
use uuid::Uuid;

#[derive(Default)]
pub struct TimedStrings(pub Mutex<HashMap<String, SystemTime>>);

#[post("/save/<string>")]
pub fn save(string: &str, timed_strings: &State<TimedStrings>) -> Result<(), Status> {
    let now = SystemTime::now();
    timed_strings
        .inner()
        .0
        .lock()
        .map(|mut guard| guard.insert(string.to_owned(), now))
        .map(|_| ())
        .map_err(|_| Status { code: 500 })
}

#[get("/load/<string>")]
pub fn load(string: &str, timed_strings: &State<TimedStrings>) -> Result<Json<u64>, Status> {
    timed_strings
        .inner()
        .0
        .lock()
        .map_err(|_| Status { code: 500 })
        .map(|guard| guard.get(&string.to_owned()).map(ToOwned::to_owned))
        .and_then(|opt| opt.ok_or(Status { code: 404 }))
        .and_then(|i| i.elapsed().map_err(|_| Status { code: 500 }))
        .map(|s| Json(s.as_secs()))
}

#[post("/ulids", data = "<data>")]
pub fn ulids(data: Json<Vec<String>>) -> Result<Json<Vec<String>>, Status> {
    Ok(Json(
        data.iter()
            .map(|s| Ulid::from_string(s))
            // Go from Vec<Result<...>> to Result<Vec<...>, ...>
            // Uses the below trait
            .verify(|r| r.is_ok(), Status { code: 400 })?
            .map(|r| r.unwrap())
            .map(|u| u.to_bytes())
            .map(|b| Uuid::from_bytes(b))
            .map(|u| format!("{}", u.hyphenated()))
            .rev()
            .collect::<Vec<_>>(),
    ))
}

const WEEK_MS: u64 = 604800000;
const DAY_MS: u64 = 86400000;
const MS_IN_SECONDS: u64 = 1000;
const THURSDAY_NUM: u64 = 3;
const DAYS_IN_WEEK: u64 = 7;

#[derive(Serialize, Default, Debug)]
#[serde(crate = "rocket::serde")]
pub struct UlidReturnData {
    #[serde(rename = "christmas eve")]
    christmas_eve: usize,
    weekday: usize,
    #[serde(rename = "in the future")]
    future: usize,
    #[serde(rename = "LSB is 1")]
    lsb1: usize,
}

struct IntermediaryData {
    is_christmas_eve: bool,
    weekday: u8,
    future: bool,
    lsb1: bool,
}

#[post("/ulids/<weekday>", data = "<data>")]
pub fn ulids_weekday(weekday: u8, data: Json<Vec<&str>>) -> Result<Json<UlidReturnData>, Status> {
    dbg!(weekday, &data);
    Ok(Json(
        data.iter()
            .map(|s| Ulid::from_string(s))
            // Go from Vec<Result<...>> to Result<Vec<...>, ...>
            // Uses the below trait
            .verify(|r| r.is_ok(), Status { code: 400 })?
            .map(|r| r.unwrap())
            .map(|u| IntermediaryData {
                // If this is ever None, I stg it's your fault and I'm too lazy to propagate this error right now
                is_christmas_eve: dbg!(DateTime::<Utc>::from_timestamp(
                    dbg!(u.timestamp_ms() / MS_IN_SECONDS) as i64,
                    0
                )
                .map(|dt| (dt.month(), dt.day())))
                    == Some((12, 24)),
                // 0 ms is the start of a Thursday in Unix timestamp.
                weekday: ((u.timestamp_ms() % WEEK_MS / DAY_MS + THURSDAY_NUM) % DAYS_IN_WEEK)
                    as u8,
                future: u.datetime().elapsed().is_err(),
                lsb1: u.0 & 0b1u128 == 1,
            })
            .fold(UlidReturnData::default(), |r, i| UlidReturnData {
                christmas_eve: r.christmas_eve + i.is_christmas_eve as usize,
                weekday: r.weekday + (i.weekday == weekday) as usize,
                future: r.future + i.future as usize,
                lsb1: r.lsb1 + i.lsb1 as usize,
            }),
    ))
}

trait Verify<E, T>
where
    Self: Sized,
{
    fn verify<F>(self, f: F, e: E) -> Result<Self, E>
    where
        F: Fn(T) -> bool;
}

impl<T, E, I> Verify<E, I> for T
where
    Self: Iterator<Item = I> + Clone + std::fmt::Debug,
{
    #[inline]
    fn verify<F>(self, f: F, e: E) -> Result<Self, E>
    where
        F: Fn(I) -> bool,
    {
        if self.clone().all(f) {
            Ok(self)
        } else {
            Err(e)
        }
    }
}
