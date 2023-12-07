use rocket::{*, serde::json::Json};
use ::serde::Serialize;

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct DaySixResp {
    elf: usize,
    #[serde(rename = "elf on a shelf")]
    elf_on_a_shelf: usize,
    #[serde(rename = "shelf with no elf on it")]
    lonely_shelves: usize,
}

#[post("/", data="<data>")]
pub fn endpoint(data: &str) -> Json<DaySixResp> {
    let mut elves = 0usize;
    let mut shelves = 0usize;
    let mut elves_on_shelves = 0usize;

    data.split("elf").for_each(|s| {
        elves += 1;
        if s.ends_with("sh") { shelves += 1; }
        if s.trim() == "on a sh" { elves_on_shelves += 1; }
    });
    
    Json(DaySixResp {
        elf: elves - 1, // Counts one extra.
        elf_on_a_shelf: elves_on_shelves,
        lonely_shelves: shelves - elves_on_shelves,
    })
}