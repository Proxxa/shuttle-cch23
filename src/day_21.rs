use std::ops::{Deref, DerefMut};

use anyhow::anyhow;
use reqwest::Client;
use rocket::{http::Status, request::FromParam, *};
use shuttle_secrets::SecretStore;

pub struct BinaryRep<T>(pub T);

impl<T> Deref for BinaryRep<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for BinaryRep<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'r> FromParam<'r> for BinaryRep<u64> {
    type Error = anyhow::Error;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        let error = Err(anyhow!(
            "Parameter must be a binary representation of a(n) {} ({} bits).",
            std::any::type_name::<u64>(),
            std::mem::size_of::<u64>()
        ));

        if !(param.len() == 64 && param.chars().all(|c| c == '0' || c == '1')) {
            error?
        }

        u64::from_str_radix(param, 2)
            .map(|b| BinaryRep(b))
            .map_err(|e| anyhow!(e))
    }
}

#[get("/coords/<bin>")]
pub fn coords(bin: BinaryRep<u64>) -> String {
    info!("CELLID: {:64b}", bin.0);

    let pt = s2::point::Point::from(s2::cellid::CellID(bin.0));
    let lat = pt.latitude().deg();
    let lon = pt.longitude().deg();

    #[derive(PartialEq, Eq)]
    enum Polarity {
        Positive,
        Negative,
    }

    fn f64_deg_to_dms_triple(f: f64) -> (Polarity, f64, f64, f64) {
        // Very unlikely that we have exactly 0.
        let pol = if f > 0. {
            Polarity::Positive
        } else {
            Polarity::Negative
        };

        let f = f.abs();
        let whole = f.floor();
        let tenth_part = (f - whole) * 60.;
        let tenth = tenth_part.floor();
        let rest = (tenth_part - tenth) * 60.;

        info!("{f} is {whole}, {tenth}, {rest}");

        (pol, whole, tenth, rest)
    }

    let (lat_p, lat_0, lat_1, lat_2) = f64_deg_to_dms_triple(lat);
    let (lon_p, lon_0, lon_1, lon_2) = f64_deg_to_dms_triple(lon);

    let out = format!(
        "{:.0}°{:.0}'{:.3}''{} {:.0}°{:.0}'{:.3}''{}",
        lat_0,
        lat_1,
        lat_2,
        if lat_p == Polarity::Positive {
            'N'
        } else {
            'S'
        },
        lon_0,
        lon_1,
        lon_2,
        if lon_p == Polarity::Positive {
            'E'
        } else {
            'W'
        }
    );

    info!("\tOUTPUT: {out}");

    out
}

#[get("/country/<bin>")]
pub async fn country(
    bin: BinaryRep<u64>,
    secrets: &State<SecretStore>,
    client: &State<Client>,
) -> (Status, Result<String, String>) {
    let pt = s2::point::Point::from(s2::cellid::CellID(bin.0));
    let lat = pt.latitude().deg();
    let lon = pt.longitude().deg();

    const ENDPOINT: &str = "https://api.opencagedata.com/geocode/v1/json";
    (Status::Ok, Ok(client
        .get(format!(
            "{ENDPOINT}?key={}&q={lat}%2C{lon}&pretty=1&no_annotations=1",
            secrets.get("OPENCAGE_KEY").unwrap()
        ))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap()
        .get("results")
        .unwrap()
        .as_array()
        .unwrap()
        .get(0)
        .unwrap()
        .get("components")
        .unwrap()
        .get("country")
        .unwrap()
    .as_str().unwrap().to_owned()))
}
