use chrono::{DateTime, Utc};
use geo_types::{Coordinate, Point};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;

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

impl Eq for TrajectoryPoint {}

impl PartialOrd<Self> for TrajectoryPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.timestamp.partial_cmp(&other.timestamp)
    }
}

impl Ord for TrajectoryPoint {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}

impl From<TrajectoryPoint> for Coordinate<f64> {
    fn from(tp: TrajectoryPoint) -> Self {
        tp.point.0
    }
}

#[derive(Serialize)]
pub struct UserTrajectory {
    pub user_id: u64,
    pub user_name: String,
    pub user_screen_name: String,

    /// chronologically sorted points
    pub points: BinaryHeap<Reverse<TrajectoryPoint>>,
}

#[cfg(test)]
mod tests {
    use super::TrajectoryPoint;
    use chrono::{DateTime, NaiveDateTime, Utc};
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;

    #[test]
    fn trajectory_point_binheap_chronological() {
        let p1 = TrajectoryPoint {
            point: Default::default(),
            timestamp: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(10, 0), Utc),
        };
        let p2 = TrajectoryPoint {
            point: Default::default(),
            timestamp: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(20, 0), Utc),
        };
        let mut binheap = BinaryHeap::new();
        binheap.push(Reverse(p1.clone()));
        binheap.push(Reverse(p2.clone()));
        assert_eq!(binheap.pop(), Some(Reverse(p1)));
        assert_eq!(binheap.pop(), Some(Reverse(p2)));
    }
}
