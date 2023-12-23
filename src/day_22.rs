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
pub struct StarCoords(f32, f32, f32);

impl StarCoords {
    pub fn distance(&self, other: &StarCoords) -> f32 {
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
pub struct Universe(Vec<StarCoords>, Vec<Portal>);

#[async_trait]
impl<'r> FromData<'r> for Universe {
    type Error = <String as FromData<'r>>::Error;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self> {
        <String as FromData<'r>>::from_data(req, data)
            .await
            .map(|s| {
                s.lines()
                    .take(1)
                    .exactly_one()
                    .unwrap()
                    .parse()
                    .map(|n_stars| {
                        Universe(
                            s.lines()
                                .skip(1)
                                .take(n_stars)
                                .map(|coords| coords.split_ascii_whitespace())
                                .map(|coords| coords.collect_tuple())
                                .map(|opt| opt.unwrap())
                                .map(|(x, y, z)| {
                                    StarCoords(
                                        x.parse().unwrap(),
                                        y.parse().unwrap(),
                                        z.parse().unwrap(),
                                    )
                                })
                                .collect_vec(),
                            s.lines()
                                .skip(2 + n_stars)
                                .map(|coords| coords.split_ascii_whitespace())
                                .map(|coords| coords.collect_tuple())
                                .map(|opt| opt.unwrap())
                                .map(|(a, b)| Portal(a.parse().unwrap(), b.parse().unwrap()))
                                .collect_vec(),
                        )
                    })
                    .unwrap()
            })
    }
}

#[post("/rocket", data = "<data>")]
pub fn flight(data: Universe) -> Option<String> {
    let Universe(star_coords, portals) = data;
    let graph = Portals::from(portals.clone());
    let n_stars = star_coords.len();

    dbg!(&graph);

    fn find_path<'a>(
        graph: &Portals,
        star: usize,
        target: usize,
        stars_visited: Vec<usize>,
    ) -> Vec<Vec<Portal>> {
        let stars_visited = stars_visited.into_iter().chain([star]);
        if star == target {
            return Vec::from([stars_visited
                .into_iter()
                .tuple_windows()
                .map(|(a, b)| Portal(a, b))
                .collect()]);
        }

        let mut paths: Vec<Vec<Portal>> = Vec::new();
        for &next in graph
            .get(&star)
            .unwrap()
            .iter()
            .filter(|&&pl| !stars_visited.clone().any(|p| p == pl))
        {
            paths.extend(find_path(
                graph,
                next,
                target,
                stars_visited.clone().collect_vec(),
            ))
        }

        dbg!(paths)
    }

    find_path(&graph, 0, n_stars - 1, Vec::new())
        .into_iter()
        .min_by_key(|path| path.len())
        .map(|path| {
            format!(
                "{} {:.3}",
                path.len(),
                path.iter().fold(0., |tot, p| tot
                    + star_coords[p.0].distance(&star_coords[p.1]))
            )
        })
}
