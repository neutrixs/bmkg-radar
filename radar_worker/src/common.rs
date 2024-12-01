#[derive(Copy, Clone, Debug)]
pub struct Coordinate {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Copy, Clone, Debug)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

pub enum Distance {
    KM(f64),
    Degrees(f64),
}

impl Distance {
    pub fn to_km(&self) -> f64 {
        match self {
            Self::Degrees(deg) => deg / 360.0 * 40075.0,
            Self::KM(km) => *km,
        }
    }

    pub fn to_degrees(&self) -> f64 {
        match self {
            Self::KM(km) => km / 40075.0 * 360.0,
            Self::Degrees(deg) => *deg,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::common::Distance;
    use approx::assert_abs_diff_eq;
    use std::f64;

    #[test]
    fn test_distance() {
        let dist = Distance::KM(180.0);
        assert_abs_diff_eq!(dist.to_km(), 180.0, epsilon = f64::EPSILON);
        assert_abs_diff_eq!(
            dist.to_degrees(),
            1.6169681846537741,
            epsilon = f64::EPSILON
        );
        assert_abs_diff_eq!(
            Distance::Degrees(dist.to_degrees()).to_km(),
            180.0,
            epsilon = f64::EPSILON
        );
    }
}
