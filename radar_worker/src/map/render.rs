use crate::common::{Coordinate, Position};
use crate::map::util::_get_tiles_range;
use crate::map::MapImagery;
use image::{DynamicImage, GenericImage, ImageBuffer, Rgba};
use std::error::Error;

struct Approx {
    north: i32,
    west: i32,
    south: i32,
    east: i32,
}

struct Exact {
    north: f64,
    west: f64,
    south: f64,
    east: f64,
}

struct CanvasMetadata {
    range: [Position; 2],
}

impl CanvasMetadata {
    fn new(bounds: [Coordinate; 2], zoom_level: i32) -> Self {
        let range = _get_tiles_range(&bounds, zoom_level);
        Self { range }
    }

    pub fn approx(&self) -> Approx {
        Approx {
            north: self.range[0].y.floor() as i32,
            west: self.range[0].x.floor() as i32,
            south: self.range[1].y.ceil() as i32,
            east: self.range[1].x.ceil() as i32,
        }
    }

    pub fn exact(&self) -> Exact {
        Exact {
            north: self.range[0].y,
            west: self.range[0].x,
            south: self.range[1].y,
            east: self.range[1].x,
        }
    }

    pub fn rows(&self) -> i32 {
        let approx = self.approx();
        approx.south - approx.north
    }

    pub fn cols(&self) -> i32 {
        let approx = self.approx();
        approx.east - approx.west
    }
}

impl MapImagery {
    async fn prepare_images(
        &self,
        meta: &CanvasMetadata,
    ) -> Result<Vec<DynamicImage>, Box<dyn Error>> {
        /*
         TODO: the x tiles might reset to 0 at 180 longitude. account that the max x value is 2^z - 1
          in the for loop
         */
        let rows = meta.rows();
        let cols = meta.cols();
        let approx = meta.approx();

        let total_capacity = rows as usize * cols as usize;
        let mut tiles_images: Vec<DynamicImage> = Vec::with_capacity(total_capacity);
        let mut futures = Vec::with_capacity(total_capacity);

        for y in approx.north..approx.south {
            for x in approx.west..approx.east {
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

        let exact = meta.exact();
        let approx = meta.approx();
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
        let approx = meta.approx();
        let exact = meta.exact();
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
}
