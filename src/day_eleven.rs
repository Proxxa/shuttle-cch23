use image::io::Reader as ImageReader;
use rocket::{
    form::{DataField, Errors, Form, FromFormField},
    http::Status,
    serde::json::Json,
    *,
};
use std::io::Cursor;

pub struct MyFormField(Vec<u8>);

#[rocket::async_trait]
impl<'v> FromFormField<'v> for MyFormField {
    async fn from_data(field: DataField<'v, '_>) -> form::Result<'v, Self> {
        field
            .data
            .open(rocket::data::ToByteUnit::bytes(usize::MAX))
            .into_bytes()
            .await
            .map(|a| MyFormField(a.into_inner()))
            .map_err(|e| {
                let mut es = Errors::new();
                es.push(e.into());
                es
            })
    }
}

#[post("/red_pixels", format = "multipart/form-data", data = "<data>")]
pub async fn bull_mode(data: Form<MyFormField>) -> Result<Json<usize>, Status> {
    ImageReader::new(Cursor::new(data.into_inner().0))
        .with_guessed_format()
        .map_err(|_| Status { code: 422 })
        .map(ImageReader::decode)
        .and_then(|res| res.map_err(|_| Status { code: 422 }))
        .map(|im| im.into_rgb16())
        .map(|buf| {
            buf.chunks_exact(3).fold(0u128, |a, p| {
                a + if p[0] as u32 > p[2] as u32 + p[1] as u32 {
                    1
                } else {
                    0
                }
            })
        })
        .map(|n| Json(n as usize))
}
