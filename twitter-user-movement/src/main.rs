mod algo;
mod model;
mod tweet;

use crate::algo::SortChronologically;
use crate::algo::Speed;
use crate::model::{MovementPoint, UserMovement};
use crate::tweet::Tweet;
use clap::{Args, Parser, Subcommand};
use eyre::Report;
use geo_types::{Coordinate, LineString};
use geojson::{Feature, FeatureCollection, GeoJson, Value};
use serde::Deserialize;
use serde_json::{json, to_value, Map};
use std::collections::hash_map::{Entry, RandomState};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use uom::si::velocity::kilometer_per_hour;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Convert JSONL-files containing tweets to a custom JSON file containing the movements for each user.
    ///
    /// The JSON will be written to stdout
    ToMovementJson(FileList),
    /// Convert JSONL-files containing tweets to a GeoJSON FeatureCollection containing a LineString for each user.
    ///
    /// The JSON will be written to stdout
    ToGeoJson(FileList),
}

#[derive(Args, Debug)]
struct FileList {
    /// JSONL files containing tweets
    jsonl_files: Vec<String>,
}

type Movements = HashMap<u64, UserMovement>;

fn main() -> eyre::Result<()> {
    let args = Cli::parse();

    match &args.command {
        Command::ToGeoJson(file_list) => {
            let movements = parse_movements(&file_list.jsonl_files)?;
            save_geojson(movements)?;
        }
        Command::ToMovementJson(file_list) => {
            let movements = parse_movements(&file_list.jsonl_files)?;
            println!("{}", serde_json::to_string(&movements)?);
        }
    }
    Ok(())
}

fn parse_movements(jsonl_files: &[String]) -> eyre::Result<Movements> {
    let mut movements = Movements::new();

    let mut buf = String::new();
    for jsonl_filename in jsonl_files.iter() {
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
                let movement_point = MovementPoint {
                    point,
                    timestamp: tweet.created_at,
                };
                match movements.entry(tweet.user.id) {
                    Entry::Occupied(mut occ) => {
                        occ.get_mut().points.push(movement_point);
                    }
                    Entry::Vacant(vac) => {
                        vac.insert(UserMovement {
                            user_id: tweet.user.id,
                            user_name: tweet.user.name,
                            user_screen_name: tweet.user.screen_name,
                            points: vec![movement_point],
                        });
                    }
                }
            }
        }
    }

    // remove all with less than two points
    movements.retain(|_, v| v.points.len() >= 2);

    // sort by time
    movements.iter_mut().for_each(|(_, v)| {
        v.points.sort_chronologically();
    });
    Ok(movements)
}

fn save_geojson(user_movements: HashMap<u64, UserMovement>) -> eyre::Result<()> {
    let mut features = Vec::with_capacity(user_movements.len());
    for (_, user_movement) in user_movements {
        let coordinates: Vec<Coordinate<f64>> = user_movement
            .points
            .iter()
            .map(|tp| tp.clone().into())
            .collect();
        let linestring = LineString::from(coordinates);

        let metrics = user_movement.metrics();

        let mut props = Map::new();
        props.insert("sp_pc_10".to_string(), to_value(metrics.speeds_kmh_pc_10)?);
        props.insert("sp_pc_50".to_string(), to_value(metrics.speeds_kmh_pc_50)?);
        props.insert("sp_pc_80".to_string(), to_value(metrics.speeds_kmh_pc_80)?);
        props.insert(
            "sp_pc_100".to_string(),
            to_value(metrics.speeds_kmh_pc_100)?,
        );
        props.insert(
            "straightness_median".to_string(),
            to_value(metrics.straightness_median)?,
        );

        props.insert(
            "max_speed_kmh".to_string(),
            to_value(
                user_movement
                    .points
                    .speed_max()
                    .map(|v| v.get::<kilometer_per_hour>()),
            )?,
        );
        props.insert("user_name".to_string(), to_value(user_movement.user_name)?);
        props.insert("user_id".to_string(), to_value(user_movement.user_id)?);
        props.insert(
            "user_screen_name".to_string(),
            to_value(user_movement.user_screen_name)?,
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
