use crate::common::{Coordinate, Distance};
use std::collections::HashMap;
use time::Duration;

const DEFAULT_THRESHOLD: Duration = Duration::minutes(20);
pub const DEFAULT_RANGE: Distance = Distance::KM(240.0);

pub struct RadarImagery {
    bounds: [Coordinate; 2],
    age_threshold: Duration,
    enforce_age_threshold: bool,
    omit_radar: Vec<String>,
    range_override: HashMap<String, Distance>,
    priority_override: HashMap<String, i32>,
}

struct RadarImageryBuilder {
    bounds: [Coordinate; 2],
    age_threshold: Option<Duration>,
    enforce_age_threshold: Option<bool>,
    omit_radar: Option<Vec<String>>,
    range_override: Option<HashMap<String, Distance>>,
    priority_override: Option<HashMap<String, i32>>,
}

impl RadarImageryBuilder {
    fn new(bounds: [Coordinate; 2]) -> Self {
        Self {
            bounds,
            age_threshold: None,
            enforce_age_threshold: None,
            omit_radar: None,
            range_override: None,
            priority_override: None,
        }
    }

    pub fn age_threshold(mut self, age_threshold: Duration) -> Self {
        self.age_threshold = Some(age_threshold);
        self
    }

    pub fn enforce_age_threshold(mut self, enforce_age_threshold: bool) -> Self {
        self.enforce_age_threshold = Some(enforce_age_threshold);
        self
    }

    pub fn omit_radar(mut self, omit_radar: Vec<String>) -> Self {
        self.omit_radar = Some(omit_radar);
        self
    }

    pub fn range_override(mut self, range_override: HashMap<String, Distance>) -> Self {
        self.range_override = Some(range_override);
        self
    }

    pub fn priority_override(mut self, priority_override: HashMap<String, i32>) -> Self {
        self.priority_override = Some(priority_override);
        self
    }

    pub fn build(self) -> RadarImagery {
        RadarImagery {
            bounds: self.bounds,
            age_threshold: self.age_threshold.unwrap_or(DEFAULT_THRESHOLD),
            enforce_age_threshold: self.enforce_age_threshold.unwrap_or_default(),
            omit_radar: self.omit_radar.unwrap_or_default(),
            range_override: self.range_override.unwrap_or_default(),
            priority_override: self.priority_override.unwrap_or_default(),
        }
    }
}
