pub mod radar_data;
pub mod render;
mod formula;
mod images_fetch;
mod radar_overlap;
mod image_crop;
mod color_scheme;

use crate::common::{Coordinate, Distance};
use crate::radar::radar_data::Legends;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use image::RgbaImage;
use std::collections::HashMap;
use time::Duration;

const DEFAULT_THRESHOLD: Duration = Duration::minutes(20);
pub const DEFAULT_RANGE: Distance = Distance::KM(240.0);
pub const DEFAULT_PRIORITY: i32 = 0;

#[derive(Clone, Debug)]
pub(crate) struct RadarImagesData {
    time: DateTime<Utc>,
    url: String,
}

#[derive(Clone, Debug)]
pub struct RadarData {
    bounds: [Coordinate; 2],
    city: String,
    station: String,
    code: String,
    center: Coordinate,
    range: Distance,
    priority: i32,
    images: Vec<RadarImagesData>,
    legends: Legends,
}

pub(crate) struct Image {
    data: RadarData,
    image: Bytes,
}

pub struct RadarImagery {
    bounds: [Coordinate; 2],
    age_threshold: Duration,
    enforce_age_threshold: bool,
    omit_radar: Vec<String>,
    ranges: HashMap<String, Distance>,
    priorities: HashMap<String, i32>,
}

pub struct RadarImageryBuilder {
    bounds: [Coordinate; 2],
    age_threshold: Option<Duration>,
    enforce_age_threshold: Option<bool>,
    omit_radar: Option<Vec<String>>,
}

pub struct RenderResult {
    pub used_radars: Vec<RadarData>,
    pub image: RgbaImage,
}

impl RadarImagery {
    pub fn builder(bounds: [Coordinate; 2]) -> RadarImageryBuilder {
        RadarImageryBuilder::new(bounds)
    }
}

impl RadarImageryBuilder {
    fn new(bounds: [Coordinate; 2]) -> Self {
        Self {
            bounds,
            age_threshold: None,
            enforce_age_threshold: None,
            omit_radar: None,
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

    pub fn build(self) -> RadarImagery {
        let mut ranges: HashMap<String, Distance> = HashMap::new();
        ranges.insert("PWK".to_string(), Distance::KM(80.));
        ranges.insert("CGK".to_string(), Distance::KM(85.));
        ranges.insert("JAK".to_string(), Distance::KM(240.));

        let mut priorities: HashMap<String, i32> = HashMap::new();
        priorities.insert("PWK".to_string(), 1);
        priorities.insert("CGK".to_string(), 1);

        RadarImagery {
            bounds: self.bounds,
            age_threshold: self.age_threshold.unwrap_or(DEFAULT_THRESHOLD),
            enforce_age_threshold: self.enforce_age_threshold.unwrap_or_default(),
            omit_radar: self.omit_radar.unwrap_or_default(),
            ranges,
            priorities,
        }
    }
}
