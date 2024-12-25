use crate::common::Distance;
use crate::radar::{Image, RadarData, RadarImagery};
use chrono::Utc;
use time::Duration;

impl RadarImagery {
    pub(crate) fn overlapping_radars(&self, radars: &Vec<Image>, radar: &Image) -> Vec<RadarData> {
        let mut overlapping = Vec::new();

        for current in radars {
            if current.data.code == radar.data.code {
                continue;
            }

            if !is_overlapping(&radar.data, &current.data) {
                continue;
            }

            if radar.data.priority > current.data.priority {
                continue;
            }

            let image_time = current.data.images.last().unwrap().time;
            let elapsed = Utc::now() - image_time;
            let elapsed = Duration::seconds(elapsed.num_seconds());

            let mut cloned_current = current.data.clone();
            if elapsed > self.age_threshold && self.enforce_age_threshold {
                cloned_current.priority = -1;
            }

            overlapping.push(cloned_current);
        }

        overlapping
    }
}

fn is_overlapping(a: &RadarData, b: &RadarData) -> bool {
    let dx = a.center.lon - b.center.lon;
    let dy = a.center.lat - b.center.lat;
    let dist = Distance::Degrees((dx * dx + dy * dy).sqrt());

    if (a.code == b.code) {
        return false;
    }

    dist.to_degrees() < a.range.to_degrees() + b.range.to_degrees()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::Coordinate;

    #[test]
    fn test_overlap() {
        // bounds does not matter here
        // the things that matter are the center and the range
        let a = RadarData {
            bounds: [Coordinate { lat: 0., lon: 0. }, Coordinate { lat: 0., lon: 0. }],
            city: "".to_string(),
            station: "".to_string(),
            code: "".to_string(),
            center: Coordinate {
                lat: 0.,
                lon: 0.,
            },
            range: Distance::Degrees(0.5),
            priority: 0,
            images: vec![],
        };

        let mut b = a.clone();
        b.code = String::from("different");
        b.center = Coordinate {
            lat: 0.,
            lon: 1.1,
        };
        assert!(!is_overlapping(&a, &b));

        b.center = Coordinate {
            lat: 0.,
            lon: 0.9,
        };
        assert!(is_overlapping(&a, &b));

        b.center = Coordinate {
            lat: 0.7,
            lon: 0.7,
        };
        assert!(is_overlapping(&a, &b));

        b.center = Coordinate {
            lat: 0.8,
            lon: 0.8,
        };
        assert!(!is_overlapping(&a, &b));
    }
}