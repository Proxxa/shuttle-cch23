use std::{cmp::min, collections::HashMap};

use ::serde::{Deserialize, Serialize};
use base64::Engine;
use rocket::{
    http::{CookieJar, Status},
    serde::json::Json,
    *,
};

// Purely monadic this time!

// Could be (u16) -> (Result<O,E>) -> Result<O, Status>
// However, other statuses are only returned from Option::ok_or
fn map_err422<O, E>(r: Result<O, E>) -> Result<O, Status> {
    r.map_err(|_| Status { code: 422 })
}

#[get("/decode")]
pub fn b64_decode(jar: &CookieJar) -> Result<String, Status> {
    // Get "recipe" from the cookies
    jar.get_pending("recipe")
        // Return 400 if no such cookie (cookies are input!)
        .ok_or(Status { code: 400 })
        // Get the value of the cookie
        .map(|a| a.value().to_string())
        // Decode the value
        .map(|s| base64::prelude::BASE64_STANDARD.decode(&s))
        // Map any error to 422
        .and_then(map_err422)
        // Attempt to parse to UTF-8
        .map(String::from_utf8)
        // Map any error to 422
        .and_then(map_err422)
}

#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
struct RecipePantryData {
    recipe: HashMap<String, usize>, // Regarding Task 3, Task 3 should be easy enough since we don't know what's
    pantry: HashMap<String, usize>, // in either ourselves!
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CookiesPantryData {
    cookies: usize,
    pantry: HashMap<String, usize>,
}

#[get("/bake")]
pub fn bake_cookies(jar: &CookieJar) -> Result<Json<CookiesPantryData>, Status> {
    // Get that recipe...
    jar.get_pending("recipe")
        // Let's skip ahead.
        .ok_or(Status { code: 400 })
        .map(|a| a.value().to_string())
        .map(|s| base64::prelude::BASE64_STANDARD.decode(&s))
        .and_then(map_err422)
        .map(String::from_utf8)
        .and_then(map_err422)
        // Parse this into the struct
        .map(|s| serde_json::from_str::<RecipePantryData>(&s))
        // Map errors! You get these lines by now.
        .and_then(map_err422)
        // Let's do some REALLY weird computations
        .map(|d| {
            // Based on a number of cookies (calculated further down)...
            (|n| {
                // Return data in JSON format
                Json(CookiesPantryData {
                    // Number of cookies
                    cookies: n,
                    // A clone of the pantry, mapped to have the necessary number
                    // of ingredients removed for all cookies made.
                    pantry: d
                        .pantry
                        .iter()
                        .map(|(k, &v)| (k.to_owned(), v - (d.recipe.get(k).unwrap_or(&0) * n)))
                        .collect(),
                })
            })(
                d.recipe
                    .iter()
                    // Get the possible number of cookies
                    .fold(usize::MAX, |a, (k, v)| {
                        // Least of current accumulator and (available / per_cookie)
                        min(
                            a,
                            if v != &0 {
                                d.pantry.get(k).unwrap_or(&0) / v
                            } else {
                                usize::MAX
                            },
                        )
                    }),
            )
        })
}
