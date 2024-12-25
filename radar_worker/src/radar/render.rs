use crate::common::PixelPosition;
use crate::radar::formula::{considerate_floor, min_q1_q2, q_inside, qx_circ, qx_half_dist, EqResult};
use crate::radar::images_fetch::fetch_images;
use crate::radar::{Image, RadarData, RadarImagery, RenderResult};
use image::codecs::png::PngDecoder;
use image::{DynamicImage, GenericImageView, ImageBuffer, RgbaImage};
use std::error::Error;
use std::io::Cursor;

struct RenderLoopResult {
    is_used: bool,
}

impl RadarImagery {
    pub async fn render(&self, width: u32, height: u32) -> Result<RenderResult, Box<dyn Error>> {
        let radars = self.get_radar_data().await;
        if let Err(e) = radars {
            return Err(format!("Failed to fetch radar datas: {}", e).into());
        }
        let radars = fetch_images(radars?).await;
        if let Err(e) = radars {
            return Err(format!("Error while fetching radar images: {}", e).into());
        }
        let radars = radars?;

        let mut used_radars: Vec<RadarData> = Vec::new();
        let mut canvas: RgbaImage = ImageBuffer::new(width, height);

        for radar in &radars {
            let result = self.render_each_radar(&mut canvas, &radars, radar);
            if let Err(e) = result {
                return Err(format!("Error while rendering radar {}: {}", &radar.data.code, e)
                    .into());
            }
            let result = result?;
            if result.is_used {
                used_radars.push(radar.data.clone());
            }
        }

        Ok(RenderResult {
            used_radars,
            image: canvas,
        })
    }

    fn render_each_radar(
        &self,
        canvas: &mut RgbaImage,
        radars: &Vec<Image>,
        radar: &Image,
    ) -> Result<RenderLoopResult, Box<dyn Error>> {
        let mut is_used = false;

        let decoder = PngDecoder::new(Cursor::new(&radar.image));
        if let Err(e) = decoder {
            return Err(format!("Error while decoding image: {}", e).into());
        }
        let decoder = decoder?;

        let image = DynamicImage::from_decoder(decoder);
        if let Err(e) = image {
            return Err(format!("Error while creating DynamicImage: {}", e).into());
        }
        let image = image?.to_rgba8();
        let crop_result = self.crop(&image, &radar.data, canvas.width(), canvas.height());
        let cropped_image = crop_result.image;
        let cropped_image_bounds = crop_result.cropped_bounds;
        let image_bounds = crop_result.accurate_bounds;

        let canvas_image_pos = crop_result.on_canvas_pos;
        let canvas_image_pos = [
            PixelPosition {
                x: canvas_image_pos[0].x.round() as u32,
                y: canvas_image_pos[0].y.round() as u32,
            },
            PixelPosition {
                x: canvas_image_pos[1].x.round() as u32,
                y: canvas_image_pos[1].y.round() as u32,
            }
        ];

        //TODO: striped pattern

        let overlapping = self.overlapping_radars(radars, &radar);

        for y in canvas_image_pos[0].y..canvas_image_pos[1].y {
            let latitude = self.bounds[0].lat - (y as f64 + 0.5) / (canvas.height() as f64) * (self
                .bounds[0].lat - self
                .bounds[1]
                .lat);

            let mut longitude_to_check = Vec::new();

            // here, we will check if the line is completely outside the circle
            // if it is, just skip the line
            // TODO: we can actually calculate where this circ bound starts on y to fasten the
            //  calculation
            match qx_circ(&radar.data, latitude) {
                EqResult::NoResult => continue,
                EqResult::Real(pos, neg) => {
                    longitude_to_check.extend(vec![pos, neg]);
                }
            };

            for overlapping_radar in &overlapping {
                // half dist will only be applicable if their priority is the same
                if radar.data.priority == overlapping_radar.priority {
                    let half_dist = qx_half_dist(&radar.data, overlapping_radar, latitude);
                    longitude_to_check.push(half_dist);
                }

                // qx_circ for the overlapping radar
                // will only applicable if their priority is the same or higher
                if overlapping_radar.priority < radar.data.priority {
                    continue;
                }
                match qx_circ(overlapping_radar, latitude) {
                    EqResult::NoResult => {}
                    EqResult::Real(pos, neg) => {
                        longitude_to_check.extend(vec![pos, neg]);
                    }
                };
            }

            let mut longitude_bounds: Vec<f64> = Vec::new();

            for longitude in longitude_to_check {
                let mut current_radar_circ_bound = q_inside(&radar.data, longitude, latitude);
                let overlay_bounds = min_q1_q2(&radar.data, &overlapping, longitude, latitude);
                for bound in overlay_bounds {
                    current_radar_circ_bound = current_radar_circ_bound.max(bound);
                }

                // if it equals zero, we're at the boundary
                if current_radar_circ_bound.abs() < 0.0000000001 {
                    longitude_bounds.push(longitude);
                }
            }

            longitude_bounds.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let pos_y_on_radar = (cropped_image_bounds[0].lat - latitude) /
                (cropped_image_bounds[0].lat - cropped_image_bounds[1].lat) * cropped_image
                .height() as f64;
            let pos_y_on_radar = considerate_floor(pos_y_on_radar) as u32;

            // longitude_bounds will always have an even length
            // math will be math
            let mut i = 0;
            while i < longitude_bounds.len() {
                let longitude_bound = longitude_bounds[i];
                // in case it isn't
                let longitude_bound_end = match longitude_bounds.get(i + 1) {
                    None => cropped_image_bounds[1].lon,
                    Some(lon) => *lon,
                };

                if longitude_bound > image_bounds[1].lon || longitude_bound_end < image_bounds[0].lon {
                    i += 2;
                    continue;
                }

                let lower_bound = longitude_bound.max(image_bounds[0].lon);
                let upper_bound = longitude_bound_end.min(image_bounds[1].lon);

                let lower_bound_on_canvas = (lower_bound - self.bounds[0].lon) /
                    (self.bounds[1].lon - self.bounds[0].lon) * canvas
                    .width() as f64;
                let upper_bound_on_canvas = (upper_bound - self.bounds[0].lon) /
                    (self.bounds[1].lon - self.bounds[0].lon) * canvas
                    .width() as f64;
                let lower_bound_on_canvas = lower_bound_on_canvas.round() as u32;
                let upper_bound_on_canvas = upper_bound_on_canvas.round() as u32;

                let lower_bound_on_radar = (lower_bound - cropped_image_bounds[0].lon) /
                    (cropped_image_bounds[1]
                        .lon - cropped_image_bounds[0].lon) * cropped_image.width() as f64;
                let upper_bound_on_radar = (upper_bound - cropped_image_bounds[0].lon) /
                    (cropped_image_bounds[1]
                        .lon - cropped_image_bounds[0].lon) * cropped_image.width() as f64;

                let distance_on_radar = upper_bound_on_radar - lower_bound_on_radar;
                let distance_on_canvas = (upper_bound_on_canvas - lower_bound_on_canvas) as f64;
                let calc = distance_on_radar / distance_on_canvas;

                for x in lower_bound_on_canvas..upper_bound_on_canvas {
                    let pos_x_on_radar = (x as f64 - lower_bound_on_canvas as f64) *
                        calc + lower_bound_on_radar;
                    let pos_x_on_radar = pos_x_on_radar.round() as u32;

                    let pixel = cropped_image.get_pixel(pos_x_on_radar, pos_y_on_radar);
                    canvas.put_pixel(x, y, pixel);
                    is_used = true;
                }

                i += 2;
            }
        }

        Ok(RenderLoopResult { is_used })
    }
}
