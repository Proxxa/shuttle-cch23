#![allow(dead_code)]
use ::serde::{Deserialize, Serialize};
use regex::Regex;
use rocket::{http::Status, serde::json::Json, *};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct NiceInput<'a> {
    input: &'a str,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct NiceOutput<'a> {
    result: &'a str,
}

impl<'a> NiceOutput<'a> {
    pub const NICE: Json<NiceOutput<'_>> = Json(NiceOutput::<'_> { result: "nice" });
    pub const NAUGHTY: (Status, Json<NiceOutput<'_>>) = (
        Status::BadRequest,
        Json(NiceOutput::<'_> { result: "naughty" }),
    );
}

#[post("/nice", data = "<data>")]
pub fn nice(
    data: Json<NiceInput<'_>>,
) -> Result<Json<NiceOutput<'static>>, (Status, Json<NiceOutput<'static>>)> {
    fn map_regex_error(_: regex::Error) -> (Status, Json<NiceOutput<'static>>) {
        (
            Status::InternalServerError,
            Json(NiceOutput {
                result: "Failed to create RegEx",
            }),
        )
    }

    let re1 = Regex::new(r"[aiueoAIUEO]").map_err(map_regex_error)?;
    let re2 = Regex::new(r"ab|cd|pq|xy").map_err(map_regex_error)?;

    dbg!(data.input);

    if dbg!(re1.find_iter(data.input).count()) >= 3
        && dbg!(data.input.chars().enumerate().fold(false, |a, (i, c)| {
            a || (i > 0
                && c.is_alphabetic()
                && data.input.chars().nth(i - 1).is_some_and(|c2| c == c2))
        }))
        && dbg!(!re2.is_match(data.input))
    {
        Ok(NiceOutput::NICE)
    } else {
        Err(NiceOutput::NAUGHTY)
    }
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ReasonableNiceOutput<'a> {
    result: &'a str,
    reason: &'a str,
}

impl<'a> ReasonableNiceOutput<'a> {
    pub const NICE: Json<ReasonableNiceOutput<'_>> = Json(ReasonableNiceOutput {
        result: "nice",
        reason: "that's a nice password",
    });
    pub const REGEX_ERROR: (Status, Json<ReasonableNiceOutput<'_>>) = (
        Status::InternalServerError,
        Json(ReasonableNiceOutput {
            result: "error",
            reason: "Failed to parse RegEx.",
        }),
    );
    pub const LENGTH: (Status, Json<ReasonableNiceOutput<'_>>) =
        ReasonableNiceOutput::create_error(Status::BadRequest, "8 chars");
    pub const TYPES: (Status, Json<ReasonableNiceOutput<'_>>) =
        ReasonableNiceOutput::create_error(Status::BadRequest, "more types of chars");
    pub const DIGITS: (Status, rocket::serde::json::Json<ReasonableNiceOutput<'_>>) =
        ReasonableNiceOutput::create_error(Status::BadRequest, "55555");
    pub const MATH: (Status, rocket::serde::json::Json<ReasonableNiceOutput<'_>>) =
        ReasonableNiceOutput::create_error(Status::BadRequest, "math is hard");
    pub const JOYLESS: (Status, rocket::serde::json::Json<ReasonableNiceOutput<'_>>) =
        ReasonableNiceOutput::create_error(Status::NotAcceptable, "not joyful enough");
    pub const SANDWICH: (Status, rocket::serde::json::Json<ReasonableNiceOutput<'_>>) =
        ReasonableNiceOutput::create_error(
            Status::UnavailableForLegalReasons,
            "illegal: no sandwich",
        );
    pub const UNICODE: (Status, rocket::serde::json::Json<ReasonableNiceOutput<'_>>) =
        ReasonableNiceOutput::create_error(Status::RangeNotSatisfiable, "outranged");
    pub const EMOJI: (Status, rocket::serde::json::Json<ReasonableNiceOutput<'_>>) =
        ReasonableNiceOutput::create_error(Status::UpgradeRequired, "😳");
    pub const TEAPOT: (Status, rocket::serde::json::Json<ReasonableNiceOutput<'_>>) =
        ReasonableNiceOutput::create_error(Status::ImATeapot, "not a coffee brewer");

    pub const fn create_error(
        status: Status,
        reason: &'static str,
    ) -> (Status, Json<ReasonableNiceOutput<'_>>) {
        (
            status,
            Json(ReasonableNiceOutput::<'static> {
                result: "naughty",
                reason,
            }),
        )
    }
}

#[post("/game", data = "<data>")]
pub fn game(
    data: Json<NiceInput<'_>>,
) -> Result<Json<ReasonableNiceOutput>, (Status, Json<ReasonableNiceOutput<'_>>)> {
    fn map_regex_err(_: regex::Error) -> (Status, Json<ReasonableNiceOutput<'static>>) {
        ReasonableNiceOutput::REGEX_ERROR
    }

    let re1 = Regex::new(r"[a-z]").map_err(map_regex_err)?;
    let re2 = Regex::new(r"[A-Z]").map_err(map_regex_err)?;
    let re3 = Regex::new(r"\d").map_err(map_regex_err)?;
    let re4 = Regex::new(r"\d+").map_err(map_regex_err)?;
    let re5 = Regex::new(r"j.+?o.+?y").map_err(map_regex_err)?;
    let re6 = Regex::new(r"[\u2980-\u2BFF]").map_err(map_regex_err)?;

    dbg!(data.input);

    if data.input.chars().count() < 8 {
        Err(ReasonableNiceOutput::LENGTH)
    } else if !(re1.is_match(data.input) && re2.is_match(data.input) && re3.is_match(data.input)) {
        Err(ReasonableNiceOutput::TYPES)
    } else if re3.find_iter(data.input).count() < 5 {
        Err(ReasonableNiceOutput::DIGITS)
    } else if re4.find_iter(data.input).fold(0usize, |a, s| {
        a + s
            .as_str()
            .parse::<usize>()
            .expect("Somehow can't parse digits into number")
    }) != 2023
    {
        Err(ReasonableNiceOutput::MATH)
    } else if !re5.is_match(data.input) {
        Err(ReasonableNiceOutput::JOYLESS)
    } else if !data.input.chars().enumerate().fold(false, |a, (i, c)| {
        a || (i > 1
            && c.is_alphabetic()
            && data.input.chars().nth(i - 2).is_some_and(|c2| c == c2)
            && data
                .input
                .chars()
                .nth(i - 1)
                .is_some_and(|c2| c2.is_alphabetic() && c != c2))
    }) {
        Err(ReasonableNiceOutput::SANDWICH)
    } else if !re6.is_match(data.input) {
        Err(ReasonableNiceOutput::UNICODE)
    } else if !data.input.chars().any(|c| match c {
        '\u{1F600}'...'\u{1F64F}' |  // Emoticons
        '\u{1F300}'...'\u{1F5FF}' |  // Misc Symbols and Pictographs
        '\u{1F680}'...'\u{1F6FF}' |  // Transport and Map
        '\u{2600}'...'\u{26FF}' |    // Misc symbols
        '\u{2700}'...'\u{27BF}' |    // Dingbats
        '\u{FE00}'...'\u{FE0F}' |    // Variation Selectors
        '\u{1F900}'...'\u{1F9FF}' |  // Supplemental Symbols and Pictographs
        '\u{1F1E6}'...'\u{1F1FF}' => // Flags
            true,
        _ => false
    }) {
        Err(ReasonableNiceOutput::EMOJI)
    } else if !sha256::digest(data.input).ends_with("a") {
        Err(ReasonableNiceOutput::TEAPOT)
    } else {
        Ok(ReasonableNiceOutput::NICE)
    }
}
