use crate::common::{Coordinate, Position};
use crate::map::util::_get_tiles_range;
use crate::map::MapImagery;
use image::{DynamicImage, GenericImage, ImageBuffer, Rgba};
use std::error::Error;

struct TilesIterator {
    index: usize,
    data: Vec<i32>,
}

struct Approx {
    north: i32,
    west: i32,
    south: i32,
    east: i32,
}

struct TileBounds {
    north: f64,
    west: f64,
    south: f64,
    east: f64,
}

struct CanvasMetadata {
    range: [Position; 2],
    z: i32,
}

impl Iterator for TilesIterator {
    type Item = i32;

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
            north: self.north.floor() as i32,
            west: self.west.floor() as i32,
            south: self.south.ceil() as i32,
            east: self.east.ceil() as i32,
        }
    }
}

impl CanvasMetadata {
    fn new(bounds: [Coordinate; 2], zoom_level: i32) -> Self {
        let range = _get_tiles_range(&bounds, zoom_level);
        Self {
            range,
            z: zoom_level,
        }
    }

    fn bounds(&self) -> TileBounds {
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

    fn iter_rows(&self) -> TilesIterator {
        let bounds = self.normalize().approx();
        let mut data: Vec<i32> = Vec::new();
        for y in bounds.north..bounds.south {
            data.push(y);
        }

        TilesIterator {
            index: 0,
            data,
        }
    }

    fn iter_cols(&self) -> TilesIterator {
        let bounds = self.normalize().approx();
        let max = 2i32.pow(self.z as u32);
        let mut data: Vec<i32> = Vec::new();

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

    fn rows(&self) -> i32 {
        let approx = self.normalize().approx();
        approx.south - approx.north
    }

    fn cols(&self) -> i32 {
        let approx = self.normalize().approx();
        approx.east - approx.west
    }
}

impl MapImagery {
    async fn prepare_images(
        &self,
        meta: &CanvasMetadata,
    ) -> Result<Vec<DynamicImage>, Box<dyn Error>> {
        let rows = meta.rows();
        let cols = meta.cols();

        let total_capacity = rows as usize * cols as usize;
        let mut tiles_images: Vec<DynamicImage> = Vec::with_capacity(total_capacity);
        let mut futures = Vec::with_capacity(total_capacity);

        for y in meta.iter_rows() {
            for x in meta.iter_cols() {
                futures.push(self.fetch_tile(x, y));
            }
        }

        let fetched_data = futures::future::join_all(futures).await;

        for result in fetched_data {
            tiles_images.push(result?)
        }

        Ok(tiles_images)
    }

    pub async fn render(&self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn Error>> {
        let meta = CanvasMetadata::new(self.bounds, self.zoom_level);
        let tiles_images = self.prepare_images(&meta).await?;

        // get the result of the first one as a reference
        let first_tile = tiles_images.get(0).ok_or("map tile amount is zero")?;
        let width = first_tile.width();
        let height = first_tile.height();

        let mut canvas: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::new(meta.cols() as u32 * width, meta.rows() as u32 * height);

        for (i, tile) in tiles_images.iter().enumerate() {
            let x = (i % meta.cols() as usize) as u32 * width;
            let y = (i / meta.cols() as usize) as u32 * height;
            let rgba = tile.to_rgba8();
            canvas.copy_from(&rgba, x, y)?;
        }

        // tiles_images not needed anymore
        drop(tiles_images);

        let exact = meta.bounds();
        let approx = meta.bounds().approx();
        let crop_left = (width as f64 * (exact.west - approx.west as f64)) as i32;
        let crop_top = (height as f64 * (exact.north - approx.north as f64)) as i32;
        let crop_right = (width as f64 * (approx.east as f64 - exact.east)) as i32;
        let crop_bottom = (height as f64 * (approx.south as f64 - exact.south)) as i32;

        let canvas_width = canvas.width() as i32 - crop_left - crop_right;
        let canvas_height = canvas.height() as i32 - crop_top - crop_bottom;

        let cropped_canvas = canvas
            .sub_image(
                crop_left as u32,
                crop_top as u32,
                canvas_width as u32,
                canvas_height as u32,
            )
            .to_image();

        Ok(cropped_canvas)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_canvas_metadata() {
        let bounds = [
            Coordinate {
                lat: -6.9005217,
                lon: 107.6144183,
            },
            Coordinate {
                lat: -6.9291531,
                lon: 107.679947,
            },
        ];

        let meta = CanvasMetadata::new(bounds, 15);
        let approx = meta.bounds().approx();
        let exact = meta.bounds();
        let rows = meta.rows();
        let cols = meta.cols();

        // for east and south, it is +1 because it's ceiled
        assert_eq!(approx.west, 26179);
        assert_eq!(approx.east, 26186);
        assert_eq!(approx.north, 17013);
        assert_eq!(approx.south, 17017);
        assert_eq!(rows, 4);
        assert_eq!(cols, 7);

        assert_abs_diff_eq!(exact.west, 26179.303496818, epsilon = 0.000001);
        assert_abs_diff_eq!(exact.east, 26185.268064711, epsilon = 0.000001);
        assert_abs_diff_eq!(exact.north, 17013.624785892, epsilon = 0.000001);
        assert_abs_diff_eq!(exact.south, 17016.249974678, epsilon = 0.000001);
    }

    #[test]
    fn test_iterator() {
        let bounds = [
            Coordinate {
                lat: 69.78,
                lon: 170.42,
            },
            Coordinate {
                lat: 60.84,
                lon: -155.23,
            }
        ];
        let meta = CanvasMetadata::new(bounds, 5);

        let iter_cols_ref = vec![31, 0, 1, 2];
        assert_eq!(meta.iter_cols().data, iter_cols_ref);

        let iter_rows_ref = vec![7, 8, 9];
        assert_eq!(meta.iter_rows().data, iter_rows_ref);
    }
}
