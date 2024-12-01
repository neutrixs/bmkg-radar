use crate::common::{Coordinate, Position};
use std::f64::consts::PI;

pub fn _get_tiles_range(bounds: &[Coordinate; 2], zoom_level: i32) -> [Position; 2] {
    let start = _coord_to_tile(&bounds[0], zoom_level);
    let end = _coord_to_tile(&bounds[1], zoom_level);

    [start, end]
}

pub fn _coord_to_tile(coord: &Coordinate, zoom: i32) -> Position {
    let pos = _coord_to_tile_no_pow(coord);
    let n = 2.0_f64.powf(zoom as f64);
    let x = n * pos.x;
    let y = n * pos.y;

    Position { x, y }
}

pub fn _coord_to_tile_no_pow(coord: &Coordinate) -> Position {
    let x = ((coord.lon + 180.) / 360.);
    let y_numerator = ((coord.lat * PI / 180.).tan() + 1. / (coord.lat * PI / 180.).cos()).ln();
    let y = (1. - y_numerator / PI) / 2.;

    Position { x, y }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::Coordinate;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_coord_to_tile() {
        let coord_1 = Coordinate {
            lat: 5.98,
            lon: 2.33,
        };
        let t1 = _coord_to_tile(&coord_1, 13);

        assert_abs_diff_eq!(t1.x, 4149.0204444, epsilon = 0.001);
        assert_abs_diff_eq!(t1.y, 3959.6740473, epsilon = 0.001);
    }

    #[test]
    fn test_get_tiles_range() {
        let bounds = [
            Coordinate {
                lat: 5.98,
                lon: 2.33,
            },
            Coordinate { lat: 1.0, lon: 3.0 },
        ];
        let range = _get_tiles_range(&bounds, 13);
        // tile works the same way as pixel positions
        // where going south means higher y
        assert_abs_diff_eq!(range[0].x, 4149.0204444, epsilon = 0.001);
        assert_abs_diff_eq!(range[0].y, 3959.6740473, epsilon = 0.001);
        assert_abs_diff_eq!(range[1].x, 4164.2666666, epsilon = 0.001);
        assert_abs_diff_eq!(range[1].y, 4073.2432891, epsilon = 0.001);
    }
}
