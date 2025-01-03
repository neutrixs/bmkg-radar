use crate::map::canvas_meta::{CanvasMetadata, TILE_DIMENSION};
use crate::map::MapImagery;
use image::{DynamicImage, GenericImage, ImageBuffer, Rgba};
use std::error::Error;

impl MapImagery {
    async fn prepare_images(
        &self,
        meta: &CanvasMetadata,
    ) -> Result<Vec<DynamicImage>, Box<dyn Error + Send + Sync>> {
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

    pub async fn render(&self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn Error + Send + Sync>> {
        let meta = CanvasMetadata::new(self.bounds, self.zoom_level);
        let tiles_images = self.prepare_images(&meta).await?;

        let mut canvas: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::new(meta.cols() * TILE_DIMENSION, meta.rows() *
                TILE_DIMENSION);

        for (i, tile) in tiles_images.iter().enumerate() {
            let x = (i % meta.cols() as usize) as u32 * TILE_DIMENSION;
            let y = (i / meta.cols() as usize) as u32 * TILE_DIMENSION;
            let rgba = tile.to_rgba8();
            canvas.copy_from(&rgba, x, y)?;
        }

        // tiles_images not needed anymore
        drop(tiles_images);

        let (width, height) = meta.dimensions();
        let crop = meta.get_crop();

        let cropped_canvas = canvas
            .sub_image(
                crop.left,
                crop.top,
                width,
                height,
            )
            .to_image();

        Ok(cropped_canvas)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::Coordinate;
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
