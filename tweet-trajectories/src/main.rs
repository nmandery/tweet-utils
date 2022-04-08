mod algo;
mod model;
mod tweet;

use crate::algo::SortChronologically;
use crate::algo::Speed;
use crate::model::{TrajectoryPoint, UserTrajectory};
use crate::tweet::Tweet;
use clap::Parser;
use geo_types::{Coordinate, LineString};
use geojson::{Feature, FeatureCollection, GeoJson, Value};
use serde_json::{to_value, Map};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use uom::si::velocity::kilometer_per_hour;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// JSONL files containing tweets
    jsonl_files: Vec<String>,
}

fn main() -> eyre::Result<()> {
    let args = Args::parse();

    let mut trajectories: HashMap<u64, UserTrajectory> = HashMap::new();

    let mut buf = String::new();
    for jsonl_filename in args.jsonl_files {
        let mut bufreader = BufReader::new(File::open(jsonl_filename)?);
        loop {
            buf.clear();
            let n_read = bufreader.read_line(&mut buf)?;
            if n_read == 0 {
                // EOF
                break;
            }
            let tweet: Tweet = match serde_json::from_str(&buf) {
                Ok(tweet) => tweet,
                Err(e) => {
                    eprintln!("failed to deserialize tweet - {}: {}", e, buf.trim_end());
                    continue;
                }
            };

            if let Some(point) = tweet.geo_point()? {
                let traj_point = TrajectoryPoint {
                    point,
                    timestamp: tweet.created_at,
                };
                match trajectories.entry(tweet.user.id) {
                    Entry::Occupied(mut occ) => {
                        occ.get_mut().points.push(traj_point);
                    }
                    Entry::Vacant(vac) => {
                        vac.insert(UserTrajectory {
                            user_id: tweet.user.id,
                            user_name: tweet.user.name,
                            user_screen_name: tweet.user.screen_name,
                            points: vec![traj_point],
                        });
                    }
                }
            }
        }
    }

    // remove all with less than two points
    trajectories.retain(|_, v| v.points.len() >= 2);
    // sort by time
    trajectories
        .iter_mut()
        .for_each(|(_, v)| v.points.sort_chronologically());

    save_geojson(trajectories)?;
    //println!("{}", serde_json::to_string(&trajectories)?);

    Ok(())
}

fn save_geojson(trajectories: HashMap<u64, UserTrajectory>) -> eyre::Result<()> {
    let mut features = Vec::with_capacity(trajectories.len());
    for (_, user_trajectory) in trajectories {
        let coordinates: Vec<Coordinate<f64>> = user_trajectory
            .points
            .iter()
            .map(|tp| tp.clone().into())
            .collect();
        let linestring = LineString::from(coordinates);

        let mut props = Map::new();
        props.insert(
            "max_speed_kmh".to_string(),
            to_value(
                user_trajectory
                    .points
                    .speed_max()
                    .map(|v| v.get::<kilometer_per_hour>()),
            )?,
        );
        props.insert(
            "user_name".to_string(),
            to_value(user_trajectory.user_name)?,
        );
        props.insert("user_id".to_string(), to_value(user_trajectory.user_id)?);
        props.insert(
            "user_screen_name".to_string(),
            to_value(user_trajectory.user_screen_name)?,
        );

        features.push(Feature {
            bbox: None,
            geometry: Some(geojson::Geometry::new(Value::from(&linestring))),
            id: None,
            properties: Some(props),
            foreign_members: None,
        })
    }

    let gj = GeoJson::FeatureCollection(FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    });

    println!("{}", gj);
    Ok(())
}
