use chrono::{DateTime, Utc};
use geo_types::Point;

pub mod angle;
pub mod speed;
pub mod time;

pub use angle::Angles;
pub use speed::Speed;
pub use time::SortChronologically;

pub trait PointInTime {
    fn timestamp(&self) -> DateTime<Utc>;
    fn point(&self) -> Point<f64>;
}
