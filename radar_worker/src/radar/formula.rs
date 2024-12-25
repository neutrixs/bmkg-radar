// https://www.desmos.com/calculator/lgmzgbhlxd

use crate::radar::RadarData;

pub enum EqResult {
    NoResult,
    Real(f64, f64),
}

pub fn q_half_dist(a: &RadarData, b: &RadarData, x: f64, y: f64) -> f64 {
    let x1 = a.center.lon;
    let x2 = b.center.lon;

    let y1 = a.center.lat;
    let y2 = b.center.lat;

    (x - x1).powi(2) - (x - x2).powi(2) + (y - y1).powi(2) - (y - y2).powi(2)
}

pub fn q_outside(r: &RadarData, x: f64, y: f64) -> f64 {
    let x1 = r.center.lon;
    let y1 = r.center.lat;
    let r1 = r.range.to_degrees();

    -(x - x1).powi(2) - (y - y1).powi(2) + r1.powi(2)
}

pub fn q_inside(r: &RadarData, x: f64, y: f64) -> f64 {
    let x1 = r.center.lon;
    let y1 = r.center.lat;
    let r1 = r.range.to_degrees();

    (x - x1).powi(2) + (y - y1).powi(2) - r1.powi(2)
}


pub fn qx_half_dist(a: &RadarData, b: &RadarData, y: f64) -> f64 {
    let x1 = a.center.lon;
    let x2 = b.center.lon;

    let y1 = a.center.lat;
    let y2 = b.center.lat;

    let numerator = -(y - y1).powi(2) + (y - y2).powi(2) - x1.powi(2) + x2.powi(2);
    let denominator = 2.0 * (x2 - x1);

    numerator / denominator
}


// this is based on the equation of the radar's circle
// isolating x
// if the current line is outside of the circle, this will return None
pub fn qx_circ(r: &RadarData, y: f64) -> EqResult {
    let x1 = r.center.lon;
    let y1 = r.center.lat;
    let r1 = r.range.to_degrees();

    let det = r1.powi(2) - (y - y1).powi(2);
    if det < 0.0 {
        return EqResult::NoResult;
    }

    let det_sqrt = det.sqrt();

    let pos = x1 + det_sqrt;
    let neg = x1 - det_sqrt;

    EqResult::Real(pos, neg)
}

pub fn min_q1_q2(radar: &RadarData, overlapping: &Vec<RadarData>, lon: f64, lat: f64) -> Vec<f64> {
    let mut result = Vec::new();
    for overlapping_radar in overlapping {
        if radar.priority > overlapping_radar.priority {
            continue;
        }

        let mut circ2_bound = q_outside(overlapping_radar, lon, lat);
        // half distance will only be applicable if the priority is the same
        if overlapping_radar.priority == radar.priority {
            let half_dist = q_half_dist(radar, overlapping_radar, lon, lat);
            circ2_bound = circ2_bound.min(half_dist);
        }

        result.push(circ2_bound);
    }

    result
}

pub fn considerate_floor(x: f64) -> f64 {
    let epsilon = f64::EPSILON * x.abs();

    if (x - x.round()).abs() < epsilon {
        x.round()
    } else {
        x.floor()
    }
}