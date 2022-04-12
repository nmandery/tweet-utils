use crate::algo::PointInTime;

pub trait SortChronologically {
    fn sort_chronologically(&mut self);
}

impl<PIT> SortChronologically for [PIT]
where
    PIT: PointInTime,
{
    fn sort_chronologically(&mut self) {
        self.sort_unstable_by(|pit1, pit2| pit1.timestamp().cmp(&pit2.timestamp()))
    }
}

#[cfg(test)]
mod tests {
    use super::SortChronologically;
    use crate::algo::PointInTime;
    use chrono::{DateTime, NaiveDateTime, Utc};
    use geo_types::Point;

    #[derive(Clone, PartialEq, Debug)]
    struct MyPit {
        p: Point<f64>,
        ts: DateTime<Utc>,
    }

    impl PointInTime for MyPit {
        fn timestamp(&self) -> DateTime<Utc> {
            self.ts
        }

        fn point(&self) -> Point<f64> {
            self.p
        }
    }

    #[test]
    fn trajectory_point_sort_chronological() {
        let p1 = MyPit {
            p: Default::default(),
            ts: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(10, 0), Utc),
        };
        let p2 = MyPit {
            p: Default::default(),
            ts: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(20, 0), Utc),
        };
        let mut v = vec![p2.clone(), p1.clone()];
        v.sort_chronologically();
        assert_eq!(v[0], p1);
        assert_eq!(v[1], p2);
    }
}
