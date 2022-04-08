use crate::algo::PointInTime;
use chrono::{DateTime, Utc};
use geo_types::{Coordinate, Point};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

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
pub struct TrajectoryPoint {
    #[serde(serialize_with = "point_ser")]
    pub point: Point<f64>,
    pub timestamp: DateTime<Utc>,
}

impl From<TrajectoryPoint> for Coordinate<f64> {
    fn from(tp: TrajectoryPoint) -> Self {
        tp.point.0
    }
}

impl PointInTime for TrajectoryPoint {
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
pub struct UserTrajectory {
    pub user_id: u64,
    pub user_name: String,
    pub user_screen_name: String,

    /// chronologically sorted points
    pub points: Vec<TrajectoryPoint>,
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
