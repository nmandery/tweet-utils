use crate::algo::straightness::StraightnessChunked;
use crate::algo::PointInTime;
use crate::Speed;
use chrono::{DateTime, Utc};
use geo_types::{Coordinate, Point};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use statrs::statistics::{Data, OrderStatistics};
use uom::si::f64::Velocity;
use uom::si::velocity::kilometer_per_hour;

fn point_ser<S>(point: &Point<f64>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut state = serializer.serialize_struct("coord", 2)?;
    state.serialize_field("x", &point.0.x)?;
    state.serialize_field("y", &point.0.y)?;
    state.end()
}

#[derive(PartialEq, Serialize, Clone, Debug)]
pub struct MovementPoint {
    #[serde(serialize_with = "point_ser")]
    pub point: Point<f64>,
    pub is_exact_location: bool,
    pub timestamp: DateTime<Utc>,

    pub text: String,
    pub in_reply_to_user_id: Option<u64>,
    pub lang: Option<String>,
    pub travel_speed_from_last_tweet_kmh: Option<f64>,
}

impl From<MovementPoint> for Coordinate<f64> {
    fn from(tp: MovementPoint) -> Self {
        tp.point.0
    }
}

impl PointInTime for MovementPoint {
    #[inline]
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    #[inline]
    fn point(&self) -> Point<f64> {
        self.point
    }
}

#[derive(Serialize)]
pub struct UserMovement {
    pub user_id: u64,
    pub user_name: String,
    pub user_screen_name: String,

    /// chronologically sorted points
    pub points: Vec<MovementPoint>,
}

impl UserMovement {
    /// max speed
    ///
    /// expects the point to be sorted chronologically
    pub fn max_speed(&self) -> Option<Velocity> {
        self.points.speed_max()
    }

    /// expects the point to be sorted chronologically
    pub fn metrics(&self) -> Metrics {
        let mut speeds_kmh_data = Data::new(
            self.points
                .speeds()
                .iter()
                .map(|s| s.get::<kilometer_per_hour>())
                .filter(|s| !s.is_nan())
                .collect::<Vec<_>>(),
        );

        let coords: Vec<_> = self.points.iter().map(|tp| tp.point.0).collect();

        Metrics {
            point_count: self.points.len(),
            straightness_median: coords.straightness_chunked_median(10),
            speeds_kmh_pc_10: speeds_kmh_data.percentile(10),
            speeds_kmh_pc_50: speeds_kmh_data.percentile(50),
            speeds_kmh_pc_80: speeds_kmh_data.percentile(80),
            speeds_kmh_pc_100: speeds_kmh_data.percentile(100),
        }
    }
}

#[derive(Debug)]
pub struct Metrics {
    pub point_count: usize,
    pub straightness_median: f64,
    pub speeds_kmh_pc_10: f64,
    pub speeds_kmh_pc_50: f64,
    pub speeds_kmh_pc_80: f64,
    pub speeds_kmh_pc_100: f64,
}

impl Metrics {
    pub fn to_vec(&self) -> Vec<f64> {
        vec![
            self.point_count as f64,
            self.straightness_median,
            self.speeds_kmh_pc_10,
            self.speeds_kmh_pc_50,
            self.speeds_kmh_pc_80,
            self.speeds_kmh_pc_100,
        ]
    }
}

#[cfg(test)]
mod tests {
    use geo_types::{coord, LineString};

    #[test]
    fn angle() {
        let ls: LineString<f64> = LineString::from(vec![
            coord!(x: 10., y:10.),
            coord!(x: 10., y:20.),
            coord!(x: 13., y:22.),
        ]);

        let mut angles = Vec::with_capacity(ls.0.len());
        for coord_window in ls.0.windows(3) {
            // vectors in 2d space
            let v2d_a = (
                coord_window[0].x - coord_window[1].x,
                coord_window[0].y - coord_window[1].y,
            );
            let v2d_b = (
                coord_window[1].x - coord_window[2].x,
                coord_window[1].y - coord_window[2].y,
            );

            let magnitude_a = (v2d_a.0.powi(2) + v2d_a.1.powi(2)).sqrt();
            let magnitude_b = (v2d_b.0.powi(2) + v2d_b.1.powi(2)).sqrt();

            let angle =
                ((v2d_a.0 * v2d_b.0 + v2d_a.1 * v2d_b.1) / (magnitude_a * magnitude_b)).acos();

            angles.push(angle.to_degrees())
        }
        dbg!(angles);
    }
}
