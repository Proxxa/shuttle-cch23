use ::serde::Deserialize;
use htmlentity::entity::{encode, CharacterSet, EncodeType, ICodedDataTrait};
use rocket::{http::Status, response::content::RawHtml, serde::json::Json, *};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct HtmlContent {
    content: String,
}

#[post("/unsafe", data = "<data>")]
pub fn unsafe_html(data: Json<HtmlContent>) -> RawHtml<String> {
    RawHtml(format!(
        include_str!("html/template_html.html"),
        data.content
    ))
}

#[post("/safe", data = "<data>")]
pub fn safe_html(data: Json<HtmlContent>) -> Result<RawHtml<String>, Status> {
    encode(
        data.content.as_bytes(),
        &EncodeType::Named,
        &CharacterSet::SpecialChars,
    )
    .to_string()
    .map(|s| RawHtml(format!(include_str!("html/template_html.html"), s)))
    .map_err(|_| Status::UnprocessableEntity)
}
