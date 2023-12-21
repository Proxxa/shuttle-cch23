use rocket::{http::Status, *};

#[get("/error")]
pub fn error() -> Status {
    Status { code: 500 }
}
