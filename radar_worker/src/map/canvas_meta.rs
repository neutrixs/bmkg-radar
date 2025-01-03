use crate::common::{Coordinate, Position};
use crate::map::util::_get_tiles_range;

pub(crate) const TILE_DIMENSION: u32 = 256;

pub(crate) struct CanvasMetadata {
    range: [Position; 2],
    z: i32,
}

pub(crate) struct TilesIterator {
    index: usize,
    pub(crate) data: Vec<u32>,
}

pub(crate) struct Crop {
    pub(crate) top: u32,
    pub(crate) left: u32,
    pub(crate) bottom: u32,
    pub(crate) right: u32,
}

pub(crate) struct Approx {
    pub(crate) north: u32,
    pub(crate) west: u32,
    pub(crate) south: u32,
    pub(crate) east: u32,
}

pub(crate) struct TileBounds {
    pub(crate) north: f64,
    pub(crate) west: f64,
    pub(crate) south: f64,
    pub(crate) east: f64,
}

impl Iterator for TilesIterator {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.data.len() {
            let item = self.data[self.index];
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

impl TileBounds {
    pub fn approx(&self) -> Approx {
        Approx {
            north: self.north.floor() as u32,
            west: self.west.floor() as u32,
            south: self.south.ceil() as u32,
            east: self.east.ceil() as u32,
        }
    }
}

impl CanvasMetadata {
    pub(crate) fn new(bounds: [Coordinate; 2], zoom_level: i32) -> Self {
        let range = _get_tiles_range(&bounds, zoom_level);
        Self {
            range,
            z: zoom_level,
        }
    }

    pub(crate) fn bounds(&self) -> TileBounds {
        TileBounds {
            north: self.range[0].y,
            west: self.range[0].x,
            south: self.range[1].y,
            east: self.range[1].x,
        }
    }

    fn normalize(&self) -> TileBounds {
        let mut bounds = self.bounds();
        let tiles_x_max = 2f64.powi(self.z);
        if bounds.east < bounds.west {
            bounds.east += tiles_x_max;
        }
        // umm since y works differently than x, I surely hope it will never clip

        bounds
    }

    pub(crate) fn iter_rows(&self) -> TilesIterator {
        let bounds = self.normalize().approx();
        let mut data: Vec<u32> = Vec::new();
        for y in bounds.north..bounds.south {
            data.push(y);
        }

        TilesIterator {
            index: 0,
            data,
        }
    }

    pub(crate) fn iter_cols(&self) -> TilesIterator {
        let bounds = self.normalize().approx();
        let max = 2u32.pow(self.z as u32);
        let mut data: Vec<u32> = Vec::new();

        for mut x in bounds.west..bounds.east {
            if x >= max {
                x -= max;
            }
            data.push(x);
        }

        TilesIterator {
            index: 0,
            data,
        }
    }

    pub(crate) fn rows(&self) -> u32 {
        let approx = self.normalize().approx();
        (approx.south - approx.north) as u32
    }

    pub(crate) fn cols(&self) -> u32 {
        let approx = self.normalize().approx();
        (approx.east - approx.west) as u32
    }

    pub(crate) fn get_crop(&self) -> Crop {
        let approx = self.bounds().approx();
        let exact = self.bounds();

        Crop {
            top: ((exact.north - approx.north as f64) * TILE_DIMENSION as f64) as u32,
            left: ((exact.west - approx.west as f64) * TILE_DIMENSION as f64) as u32,
            bottom: ((approx.south as f64 - exact.south) * TILE_DIMENSION as f64) as u32,
            right: ((approx.east as f64 - exact.east) * TILE_DIMENSION as f64) as u32,
        }
    }

    pub(crate) fn dimensions(&self) -> (u32, u32) {
        let width = self.cols() * TILE_DIMENSION;
        let height = self.rows() * TILE_DIMENSION;
        let crop = self.get_crop();

        (width - crop.left - crop.right, height - crop.top - crop.bottom)
    }
}