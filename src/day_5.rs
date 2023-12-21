use rocket::{serde::json::Json, *};

#[post("/?<offset>&<limit>", data = "<data>", rank = 1)]
pub fn slicing(
    offset: Option<usize>,
    limit: Option<usize>,
    data: Json<Vec<&str>>,
) -> Json<Vec<&str>> {
    info!("data = {data:#?}");
    Json(
        data.0
            .into_iter()
            .skip(offset.unwrap_or(0))
            .take(limit.unwrap_or(usize::MAX))
            .collect(),
    )
}

// Different endpoints for different return types.
#[post("/?<offset>&<limit>&<split>", data = "<data>", rank = 0)]
pub fn splitting(
    offset: Option<usize>,
    limit: Option<usize>,
    split: usize,
    data: Json<Vec<&str>>,
) -> Json<Vec<Vec<&str>>> {
    Json(
        slicing(offset, limit, data)
            .0
            .chunks(split)
            .map(|chunk| chunk.to_vec())
            .collect(),
    )
}
