use rocket::{*, serde::json::Json, form::{Form, DataField, FromFormField, Errors}, http::Status};
use image::io::Reader as ImageReader;
use std::io::Cursor;


pub struct MyFormField(Vec<u8>);

impl<'v> FromFormField<'v> for MyFormField {
    fn from_data<'life0,'async_trait>(field:DataField<'v,'life0>) ->  core::pin::Pin<Box<dyn core::future::Future<Output = form::Result<'v,Self> > + core::marker::Send+'async_trait> >where 'v:'async_trait,'life0:'async_trait,Self:'async_trait {
        Box::pin(async { field.data.open(rocket::data::ToByteUnit::bytes(usize::MAX)).into_bytes().await.map(|a| {
            MyFormField(a.into_inner())
        }).map_err(|e| {
            let mut es = Errors::new();
            es.push(e.into());
            es
        }) })
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
                a + if p[0] as u32 > p[2] as u32 + p[1] as u32 { 1 } else { 0 }
            })
        })
        .map(|n| Json(n as usize))
}