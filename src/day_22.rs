use std::{
    collections::{HashMap, HashSet},
    ops::{Deref, DerefMut},
};

use itertools::Itertools;
use rocket::{data::FromData, response::status::BadRequest, *};

#[post("/integers", data = "<data>")]
pub fn integers(data: &str) -> Result<String, BadRequest<String>> {
    data.lines()
        .map(|s| usize::from_str_radix(s, 10))
        .try_fold(0, |a, i| i.map(|i| a ^ i))
        .map(|n| "🎁".repeat(n).to_string())
        .map_err(|e| BadRequest(e.to_string()))
}

#[derive(Debug, Copy, Clone)]
pub struct PlanetCoords(f32, f32, f32);

impl PlanetCoords {
    pub fn distance(&self, other: &PlanetCoords) -> f32 {
        f32::sqrt(
            f32::powi(self.0 - other.0, 2)
                + f32::powi(self.1 - other.1, 2)
                + f32::powi(self.2 - other.2, 2),
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Portal(usize, usize);

#[derive(Clone, Debug)]
pub struct Portals(HashMap<usize, HashSet<usize>>);

impl<T> From<T> for Portals
where
    T: IntoIterator<Item = Portal>,
{
    fn from(value: T) -> Self {
        Portals(
            value
                .into_iter()
                .fold(HashMap::new(), |mut map, Portal(from, to)| {
                    map.entry(from).or_default().insert(to);
                    map.entry(to).or_default().insert(from);
                    map
                }),
        )
    }
}

impl Deref for Portals {
    type Target = HashMap<usize, HashSet<usize>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Portals {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone)]
pub struct Universe(Vec<PlanetCoords>, Vec<Portal>);

#[async_trait]
impl<'r> FromData<'r> for Universe {
    type Error = <String as FromData<'r>>::Error;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self> {
        <String as FromData<'r>>::from_data(req, data)
            .await
            .map(|s| {
                Universe(
                    s.lines()
                        .skip(1)
                        .take(s.lines().take(1).exactly_one().unwrap().parse().unwrap())
                        .map(|coords| coords.split_ascii_whitespace())
                        .map(|coords| coords.collect_tuple())
                        .map(|opt| opt.unwrap())
                        .map(|(x, y, z)| {
                            PlanetCoords(x.parse().unwrap(), y.parse().unwrap(), z.parse().unwrap())
                        })
                        .collect_vec(),
                    s.lines()
                        .skip(
                            2 + s
                                .lines()
                                .take(1)
                                .exactly_one()
                                .unwrap()
                                .parse::<usize>()
                                .unwrap(),
                        )
                        .map(|coords| coords.split_ascii_whitespace())
                        .map(|coords| coords.collect_tuple())
                        .map(|opt| opt.unwrap())
                        .map(|(a, b)| Portal(a.parse().unwrap(), b.parse().unwrap()))
                        .collect_vec(),
                )
            })
    }
}

#[post("/rocket", data = "<data>")]
pub fn flight(data: Universe) -> Option<String> {
    println!("\n");
    info!("New universe discovered! Traversing...");
    let Universe(planet_coords, portals) = data;
    let graph = Portals::from(portals.clone());
    let n_planets = planet_coords.len();

    dbg!(&graph);

    fn find_path<'a>(
        graph: &Portals,
        planet: usize,
        target: usize,
        planets_visited: Vec<usize>,
    ) -> Vec<Vec<Portal>>
    {
        info!("Arrived at star {planet}");
        let planets_visited = planets_visited.into_iter().chain([planet]);
        if planet == target {
            info!("\tTarget reached.");
            return Vec::from([planets_visited
                .into_iter()
                .tuple_windows()
                .map(|(a, b)| Portal(a, b))
                .collect()]);
        }

        let mut paths: Vec<Vec<Portal>> = Vec::new();
        for &next in graph
            .get(&planet)
            .unwrap()
            .iter()
            .filter(|&&pl| !planets_visited.clone().any(|p| p == pl))
        {
            info!("\tTrying to traverse {planet}->{next}");
            paths.extend(find_path(graph, next, target, planets_visited.clone().collect_vec()))
        }

        dbg!(paths)
    }

    find_path(&graph, 0, n_planets - 1, Vec::new())
        .into_iter()
        .min_by_key(|path| path.len())
        .map(|path| {
            format!(
                "{} {:.3}",
                path.len(),
                path.iter().fold(0., |tot, p| tot
                    + planet_coords[p.0].distance(&planet_coords[p.1]))
            )
        })

    // todo!()
    // None
}
