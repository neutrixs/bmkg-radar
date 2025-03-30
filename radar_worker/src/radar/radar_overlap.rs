use crate::common::Distance;
use crate::radar::{Image, RadarData, RadarImagery};

impl RadarImagery {
    pub(crate) fn overlapping_radars<'a>(
        &self,
        radars: &'a Vec<Image>,
        radar: &Image,
    ) -> Vec<&'a RadarData> {
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

            overlapping.push(&current.data);
        }

        overlapping
    }
}

fn is_overlapping(a: &RadarData, b: &RadarData) -> bool {
    let dx = a.center.lon - b.center.lon;
    let dy = a.center.lat - b.center.lat;
    let dist = Distance::Degrees((dx * dx + dy * dy).sqrt());

    if a.code == b.code {
        return false;
    }

    dist.to_degrees() < a.range.to_degrees() + b.range.to_degrees()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::Coordinate;
    use crate::radar::radar_data::Legends;

    #[test]
    fn test_overlap() {
        // bounds does not matter here
        // the things that matter are the center and the range
        let a = RadarData {
            bounds: [
                Coordinate { lat: 0., lon: 0. },
                Coordinate { lat: 0., lon: 0. },
            ],
            city: "".to_string(),
            station: "".to_string(),
            code: "".to_string(),
            center: Coordinate { lat: 0., lon: 0. },
            range: Distance::Degrees(0.5),
            priority: 0,
            striped: false,
            images: vec![],
            legends: Legends {
                levels: vec![],
                colors: vec![],
            },
        };

        let mut b = a.clone();
        b.code = String::from("different");
        b.center = Coordinate { lat: 0., lon: 1.1 };
        assert!(!is_overlapping(&a, &b));

        b.center = Coordinate { lat: 0., lon: 0.9 };
        assert!(is_overlapping(&a, &b));

        b.center = Coordinate { lat: 0.7, lon: 0.7 };
        assert!(is_overlapping(&a, &b));

        b.center = Coordinate { lat: 0.8, lon: 0.8 };
        assert!(!is_overlapping(&a, &b));
    }
}
