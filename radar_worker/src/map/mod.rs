pub mod bounding;
mod fake_headers;
mod fetch_tile;
pub mod render;
mod util;

use crate::common::Coordinate;
use crate::map::util::_coord_to_tile_no_pow;
use std::env;
use std::time::Duration;
use url::Url;

const DEFAULT_MAX_TILES: i32 = 50;

#[derive(Copy, Clone, Debug)]
pub enum MapStyle {
    OpenCycleMap,
    Transport,
    Landscape,
    Outdoors,
    Atlas,
    TransportDark,
    Spinal,
    Pioneer,
    Neighborhood,
    MobileAtlas,
}

impl MapStyle {
    pub(crate) fn url(&self, x: i32, y: i32, z: i32) -> String {
        let base: &str = match self {
            Self::OpenCycleMap => "https://tile.thunderforest.com/cycle",
            Self::Transport => "https://tile.thunderforest.com/transport",
            Self::Landscape => "https://tile.thunderforest.com/landscape",
            Self::Outdoors => "https://tile.thunderforest.com/outdoors",
            Self::Atlas => "https://tile.thunderforest.com/atlas",
            Self::TransportDark => "https://tile.thunderforest.com/transport-dark",
            Self::Spinal => "https://tile.thunderforest.com/spinal-map",
            Self::Pioneer => "https://tile.thunderforest.com/pioneer",
            Self::Neighborhood => "https://tile.thunderforest.com/neighbourhood",
            Self::MobileAtlas => "https://tile.thunderforest.com/mobile-atlas",
        };

        let mut url = Url::parse(&format!("{}/{}/{}/{}.png", base, z, x, y))
            .expect("Error parsing URL");

        url.query_pairs_mut().append_pair(
            "apikey",
            &env::var("THUNDERFOREST_APIKEY").unwrap_or("".to_string()),
        );

        url.to_string()
    }
}

pub enum ZoomSetting {
    MaxTiles(i32),
    ZoomLevel(i32),
}

pub struct MapImagery {
    bounds: [Coordinate; 2],
    zoom_level: i32,
    style: MapStyle,
    timeout_duration: Duration,
}

impl MapImagery {
    pub fn builder(bounds: [Coordinate; 2]) -> MapImageryBuilder {
        MapImageryBuilder::new(bounds)
    }
}

pub struct MapImageryBuilder {
    bounds: [Coordinate; 2],
    zoom_setting: Option<ZoomSetting>,
    style: Option<MapStyle>,
    timeout_duration: Option<Duration>,
}

impl MapImageryBuilder {
    fn new(bounds: [Coordinate; 2]) -> Self {
        Self {
            bounds,
            zoom_setting: None,
            style: None,
            timeout_duration: None,
        }
    }

    fn auto_zoom_level(&self, tiles: i32) -> i32 {
        let start = _coord_to_tile_no_pow(&self.bounds[0]);
        let end = _coord_to_tile_no_pow(&self.bounds[1]);

        // based on the original formula
        // where (x1 - x0)(y1 - y0) = MAX TILES
        // here, we're solving for z

        let z = tiles as f64 / ((end.y - start.y) * (end.x - start.x));
        let z = z.ln() / 4f64.ln() + 0.5;
        z.floor() as i32
    }

    pub fn zoom_setting(mut self, zoom_setting: ZoomSetting) -> Self {
        self.zoom_setting = Some(zoom_setting);
        self
    }

    pub fn map_style(mut self, style: MapStyle) -> Self {
        self.style = Some(style);
        self
    }

    pub fn timeout_duration(mut self, timeout: Duration) -> Self {
        self.timeout_duration = Some(timeout);
        self
    }

    pub fn build(&self) -> MapImagery {
        let zoom_level = match self.zoom_setting {
            Some(ZoomSetting::ZoomLevel(zl)) => zl,
            Some(ZoomSetting::MaxTiles(tiles)) => self.auto_zoom_level(tiles),
            None => self.auto_zoom_level(DEFAULT_MAX_TILES),
        };

        MapImagery {
            bounds: self.bounds,
            zoom_level,
            style: self.style.unwrap_or_else(|| MapStyle::Transport),
            timeout_duration: self.timeout_duration.unwrap_or_else(|| Duration::from_secs(10)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_zoom_level() {
        let bounds = [
            Coordinate {
                lat: -6.8423027,
                lon: 107.5468144,
            },
            Coordinate {
                lat: -6.9675463,
                lon: 107.7356557,
            },
        ];

        let imagery = MapImageryBuilder::new(bounds)
            .zoom_setting(ZoomSetting::MaxTiles(50))
            .build();
        assert_eq!(imagery.zoom_level, 14);

        let bounds = [
            Coordinate {
                lat: -6.0871085,
                lon: 106.7597323,
            },
            Coordinate {
                lat: -7.7006901,
                lon: 108.9011875,
            },
        ];

        let imagery = MapImageryBuilder::new(bounds)
            .zoom_setting(ZoomSetting::MaxTiles(50))
            .build();
        assert_eq!(imagery.zoom_level, 10);
    }
}
