mod model;
mod tweet;

use crate::model::{TrajectoryPoint, UserTrajectory};
use crate::tweet::Tweet;
use clap::Parser;
use geo_types::{Coordinate, LineString};
use geojson::{Feature, FeatureCollection, GeoJson, Value};
use serde_json::{to_value, Map};
use std::cmp::Reverse;
use std::collections::hash_map::Entry;
use std::collections::{BinaryHeap, HashMap};
use std::fs::File;
use std::io::{BufRead, BufReader};

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
                    eprintln!(
                        "failed to deserialize tweet - {}: {}",
                        e.to_string(),
                        buf.trim_end()
                    );
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
                        occ.get_mut().points.push(Reverse(traj_point));
                    }
                    Entry::Vacant(vac) => {
                        let mut binheap = BinaryHeap::new();
                        binheap.push(Reverse(traj_point));
                        vac.insert(UserTrajectory {
                            user_id: tweet.user.id,
                            user_name: tweet.user.name,
                            user_screen_name: tweet.user.screen_name,
                            points: binheap,
                        });
                    }
                }
            }
        }
    }

    // remove all with less than two points
    trajectories.retain(|_, v| v.points.len() >= 2);

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
            .map(|tp| tp.0.clone().into())
            .collect();
        let linestring = LineString::from(coordinates);

        let mut props = Map::new();
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

    println!("{}", gj.to_string());
    Ok(())
}
