use geo::convex_hull::quick_hull;
use geo::geodesic_distance::GeodesicDistance;
use geo_types::{Coordinate, Point};
use statrs::statistics::{Data, Median};

/// describes the straightness of line using a single number
pub trait Straightness {
    /// straightness
    ///
    /// possible return values:
    /// * 1.0 -> perfectly straight
    /// * < 1.0 -> not straight
    fn straightness(&self) -> f64;
}

pub trait StraightnessChunked {
    fn straightness_chunked(&self, chunk_size: usize) -> Vec<f64>;

    fn straightness_chunked_median(&self, chunk_size: usize) -> f64 {
        Data::new(self.straightness_chunked(chunk_size)).median()
    }
}

impl Straightness for [Coordinate<f64>] {
    fn straightness(&self) -> f64 {
        let s = straightness(self);
        if s.is_nan() {
            1.0
        } else {
            s
        }
    }
}

impl StraightnessChunked for [Coordinate<f64>] {
    fn straightness_chunked(&self, chunk_size: usize) -> Vec<f64> {
        self.chunks(chunk_size)
            .map(|chunk| chunk.straightness())
            .collect()
    }
}

fn straightness(coords: &[Coordinate<f64>]) -> f64 {
    let mut coords_copy = coords.to_vec();
    geodesic_distance_covered(&quick_hull(&mut coords_copy).0)
        / 2.0
        / geodesic_distance_covered(coords)
}

fn geodesic_distance_covered(coords: &[Coordinate<f64>]) -> f64 {
    coords
        .windows(2)
        .map(|window| Point::from(window[0]).geodesic_distance(&Point::from(window[1])))
        .sum()
}
