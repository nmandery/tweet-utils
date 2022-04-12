use chrono::{DateTime, Utc};
use datetime::datefmt_de;
use geo::centroid::Centroid;
use geo_types::{Point, Polygon};
use serde::Deserialize;

mod datetime;

#[derive(Deserialize)]
pub struct Tweet {
    pub id: u64,
    pub user: User,

    #[serde(deserialize_with = "datefmt_de")]
    pub created_at: DateTime<Utc>,
    pub place: Option<Place>,
    pub coordinates: Option<geojson::Geometry>,
}

impl Tweet {
    pub fn geo_point(&self) -> eyre::Result<Option<Point<f64>>> {
        if let Some(gjg) = self.coordinates.as_ref() {
            Ok(Some(gjg.value.clone().try_into()?))
        //    } else if let Some(place) = self.place.as_ref() {
        //        let poly: Polygon<f64> = place.bounding_box.value.clone().try_into()?;
        //        Ok(poly.centroid())
        } else {
            Ok(None)
        }
    }
}

#[derive(Deserialize)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub screen_name: String,
}

#[derive(Deserialize)]
pub struct Place {
    pub bounding_box: geojson::Geometry,
}

#[cfg(test)]
mod tests {
    use crate::tweet::Tweet;
    use std::fs::File;

    #[test]
    fn parse_tweet() {
        let tweet: Tweet = serde_json::from_reader(
            File::open(format!("{}/../data/tweet.json", env!("CARGO_MANIFEST_DIR"))).unwrap(),
        )
        .unwrap();
        assert_eq!(tweet.id, 1307025659294674945);
        //assert_eq!(tweet.created_at)
        assert!(tweet.coordinates.is_some());
        assert!(tweet.geo_point().is_ok());
    }
}
