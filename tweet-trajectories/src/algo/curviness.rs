use crate::algo::angle::angle_radians;
use geo::prelude::GeodesicDistance;
use geo_types::{Coordinate, Point};
use uom::si::f64::Length;
use uom::si::length::meter;

/// curviness = angle(degrees) * length of the line segment in meters
pub trait Curviness {
    fn curviness(&self) -> Vec<f64>;

    fn curviness_total(&self) -> f64 {
        let curviness = self.curviness();
        curviness.iter().sum::<f64>() / (curviness.len() as f64)
    }
}

impl Curviness for [Coordinate<f64>] {
    fn curviness(&self) -> Vec<f64> {
        // TODO: making this an iterator

        let mut curviness = Vec::with_capacity(self.len().saturating_sub(1));
        for window in self.windows(3) {
            let angle = angle_radians(&[window[0], window[1], window[2]]);

            // consequent, duplicate points will make the returned angle NaN.
            if !angle.is_nan() {
                let length_window = geodesic_distance_covered(window);
                curviness.push(angle.to_degrees() * length_window.get::<meter>());
            }
        }
        curviness
    }
}

fn geodesic_distance_covered(coords: &[Coordinate<f64>]) -> Length {
    Length::new::<meter>(
        coords
            .windows(2)
            .map(|window| Point::from(window[0]).geodesic_distance(&Point::from(window[1])))
            .sum(),
    )
}

#[cfg(test)]
mod tests {
    use crate::algo::curviness::Curviness;
    use geo_types::coord;

    #[test]
    fn coords_curviness() {
        /*
        let coords = vec![
            coord!(x: 10., y:10.),
            coord!(x: 10., y:20.),
            coord!(x: 10., y:10.),
            coord!(x: 10., y:20.),
            coord!(x: 10., y:10.),
        ];

         */

        let coords = vec![
            coord!(x: 10., y:10.),
            coord!(x: 15., y:11.),
            coord!(x: 20., y:9.),
            coord!(x: 25., y:14.),
            coord!(x: 30., y:9.),
            coord!(x: 60., y:10.),
        ];

        dbg!(coords.curviness_total());
        dbg!(coords.curviness());
    }
}
