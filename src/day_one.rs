use std::num::ParseIntError;

use rocket::{
    http::uri::{fmt::Path, Segments},
    request::FromSegments,
    *,
};

pub struct BitXORFromSeg(pub(self) i128);

// Doing it this way means that I can let Rocket return 422 Unprocessable Entity
// for me! I'm not even unwinding any errors! Also, we LOVE not having to iterate
// or allocate multiple times. I never even allocate heap directly!
#[get("/<res..>")]
pub fn bit_sled_cube(res: BitXORFromSeg) -> String {
    res.0.pow(3).to_string()
}

impl<'r> FromSegments<'r> for BitXORFromSeg {
    type Error = ParseIntError;

    fn from_segments(segments: Segments<'r, Path>) -> Result<Self, Self::Error> {
        let mut val = 0i128;
        for seg in segments {
            val ^= seg.parse::<i128>()?;
        }

        Ok(BitXORFromSeg(val))
    }
}

