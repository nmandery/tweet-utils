use geo_types::{CoordFloat, Coordinate, LineString};
use nalgebra::{ComplexField, RealField, Vector2};

pub trait Angles {
    type AngleType;
    fn angles_radians(&self) -> Vec<Self::AngleType>;
    fn angles_degrees(&self) -> Vec<Self::AngleType>;
}

/// compute the angles between the coordinates using a three-coordinates wide sliding window.
impl<T> Angles for [Coordinate<T>]
where
    T: CoordFloat + RealField,
{
    type AngleType = T;

    fn angles_radians(&self) -> Vec<Self::AngleType> {
        angles_radians(self, |a| a)
    }

    fn angles_degrees(&self) -> Vec<Self::AngleType> {
        angles_radians(self, |a| a.to_degrees())
    }
}

impl<T> Angles for LineString<T>
where
    T: CoordFloat + RealField,
{
    type AngleType = T;

    fn angles_radians(&self) -> Vec<Self::AngleType> {
        self.0.angles_radians()
    }

    fn angles_degrees(&self) -> Vec<Self::AngleType> {
        self.0.angles_degrees()
    }
}

pub fn angle_radians<T>(coords: &[Coordinate<T>; 3]) -> T
where
    T: CoordFloat + RealField,
{
    // vectors in 2d space
    let v2d_a = Vector2::new(coords[0].x - coords[1].x, coords[0].y - coords[1].y);
    let v2d_b = Vector2::new(coords[1].x - coords[2].x, coords[1].y - coords[2].y);

    ComplexField::acos(v2d_a.dot(&v2d_b) / (v2d_a.magnitude() * v2d_b.magnitude()))
}

fn angles_radians<T, C>(coord_sequence: &[Coordinate<T>], transform: C) -> Vec<T>
where
    T: CoordFloat + RealField,
    C: Fn(T) -> T,
{
    let mut angles = Vec::with_capacity(coord_sequence.len().saturating_sub(1));
    for coord_window in coord_sequence.windows(3) {
        angles.push(transform(angle_radians(&[
            coord_window[0],
            coord_window[1],
            coord_window[2],
        ])));
    }
    angles
}

#[cfg(test)]
mod tests {
    use crate::algo::angle::Angles;
    use geo_types::{coord, LineString};

    #[test]
    fn angle_nalgebra() {
        let ls: LineString<f64> = LineString::from(vec![
            coord!(x: 10., y:10.),
            coord!(x: 10., y:20.),
            coord!(x: 18., y:20.),
        ]);
        let angles = ls.angles_degrees();
        assert_eq!(angles, vec![90.0]);
    }
}
