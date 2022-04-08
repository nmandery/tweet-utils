use geo::prelude::GeodesicDistance;
use ordered_float::OrderedFloat;
use uom::si::f64::{Length, Time, Velocity};
use uom::si::length::meter;
use uom::si::time::second;
use uom::si::velocity::meter_per_second;

use crate::algo::PointInTime;

pub fn speed<CIP>(tp1: &CIP, tp2: &CIP) -> Velocity
where
    CIP: PointInTime,
{
    let dur = tp2.timestamp() - tp1.timestamp();
    Length::new::<meter>(tp1.point().geodesic_distance(&tp2.point()))
        / Time::new::<second>(dur.num_seconds().abs() as f64)
}

pub trait Speed {
    fn speeds(&self) -> Vec<Velocity>;

    fn speed_max(&self) -> Option<Velocity> {
        self.speeds()
            .iter()
            .filter_map(|v| {
                let value = v.get::<meter_per_second>();
                if v.is_nan() {
                    None
                } else {
                    Some(OrderedFloat::from(value))
                }
            })
            .max()
            .map(|oflt| Velocity::new::<meter_per_second>(oflt.0))
    }
}

impl<CIP> Speed for [CIP]
where
    CIP: PointInTime,
{
    fn speeds(&self) -> Vec<Velocity> {
        self.windows(2)
            .map(|window| speed(&window[0], &window[1]))
            .collect()
    }
}
