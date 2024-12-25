use crate::common::{Coordinate, Position};
use crate::radar::{RadarData, RadarImagery};
use image::{GenericImageView, RgbaImage, SubImage};

pub(crate) fn cropped_bounds(
    crop: &Crop,
    uncropped_radar: &RgbaImage,
    radar: &RadarData,
) -> [Coordinate; 2] {
    let lat_distance = radar.bounds[0].lat - radar.bounds[1].lat;
    let lon_distance = radar.bounds[1].lon - radar.bounds[0].lon;
    let width = uncropped_radar.width() as f64;
    let height = uncropped_radar.height() as f64;

    [
        Coordinate {
            lat: radar.bounds[0].lat - lat_distance * crop.top / height,
            lon: radar.bounds[0].lon + lon_distance * crop.left / width,
        },
        Coordinate {
            lat: radar.bounds[1].lat + lat_distance * crop.bottom / height,
            lon: radar.bounds[1].lon - lon_distance * crop.right / width,
        },
    ]
}

#[derive(Debug)]
pub(crate) struct Crop {
    pub top: f64,
    pub left: f64,
    pub bottom: f64,
    pub right: f64,
}

impl Crop {
    fn approx(&self) -> CropApprox {
        // imagine 9.9999999999999 being floored to 9
        // not fun isn't it? makes everything worse
        // therefore it should be accounted
        let tolerance = 0.999999999;
        let mut top = self.top.floor() as u32;
        let mut left = self.left.floor() as u32;
        let mut bottom = self.bottom.floor() as u32;
        let mut right = self.right.floor() as u32;

        if self.top - top as f64 > tolerance {
            top += 1;
        }

        if self.left - left as f64 > tolerance {
            left += 1;
        }

        if self.bottom - bottom as f64 > tolerance {
            bottom += 1;
        }

        if self.right - right as f64 > tolerance {
            right += 1;
        }

        CropApprox {
            top,
            left,
            bottom,
            right,
        }
    }

    fn bounds(&self, uncropped_radar: &RgbaImage, radar: &RadarData) -> [Coordinate; 2] {
        cropped_bounds(self, uncropped_radar, radar)
    }
}

#[derive(Debug)]
pub(crate) struct CropApprox {
    pub top: u32,
    pub left: u32,
    pub bottom: u32,
    pub right: u32,
}

impl CropApprox {
    fn bounds(&self, uncropped_radar: &RgbaImage, radar: &RadarData) -> [Coordinate; 2] {
        let self_f64 = Crop {
            top: self.top as f64,
            left: self.left as f64,
            bottom: self.bottom as f64,
            right: self.right as f64,
        };
        self_f64.bounds(uncropped_radar, radar)
    }
}

pub(crate) struct CropResult<'a> {
    pub image: SubImage<&'a RgbaImage>,
    pub cropped_bounds: [Coordinate; 2],
    pub accurate_bounds: [Coordinate; 2],
    pub on_canvas_pos: [Position; 2],
}

impl RadarImagery {
    pub(crate) fn crop<'a>(
        &self,
        image: &'a RgbaImage,
        radar: &RadarData,
        canvas_width: u32,
        canvas_height: u32,
    ) -> CropResult<'a> {
        let width = image.width() as f64;
        let height = image.height() as f64;

        let bounds_width = radar.bounds[1].lon - radar.bounds[0].lon;
        let bounds_height = radar.bounds[0].lat - radar.bounds[1].lat;

        let relative_crop_left =
            ((self.bounds[0].lon - radar.bounds[0].lon) / bounds_width).max(0.);
        let relative_crop_right =
            ((radar.bounds[1].lon - self.bounds[1].lon) / bounds_width).max(0.);
        let relative_crop_top =
            ((radar.bounds[0].lat - self.bounds[0].lat) / bounds_height).max(0.);
        let relative_crop_bottom =
            ((self.bounds[1].lat - radar.bounds[1].lat) / bounds_height).max(0.);

        let crop = Crop {
            left: relative_crop_left * width,
            right: relative_crop_right * width,
            top: relative_crop_top * height,
            bottom: relative_crop_bottom * height,
        };
        let crop_approx = crop.approx();
        let bounds = crop.bounds(&image, radar);
        let bounds_approx = crop_approx.bounds(&image, radar);

        // crop image to the map's bound (if needed)
        // we will not resize at all here, instead
        // when putting the pixels, we need to find which pixel is the closest
        // exactly the same as FilterType::Nearest
        let image = image
            .view(
                crop_approx.left,
                crop_approx.top,
                image.width() - crop_approx.left - crop_approx.right,
                image.height() - crop_approx.top - crop_approx.bottom,
            );

        // this should be the exact position
        let on_canvas_start = Position {
            x: (bounds[0].lon - self.bounds[0].lon)
                / (self.bounds[1].lon - self.bounds[0].lon)
                * canvas_width as f64,
            y: (self.bounds[0].lat - bounds[0].lat)
                / (self.bounds[0].lat - self.bounds[1].lat)
                * canvas_height as f64,
        };

        let on_canvas_end = Position {
            x: (bounds[1].lon - self.bounds[0].lon) /
                (self.bounds[1].lon - self.bounds[0].lon) * canvas_width as f64,
            y: (self.bounds[0].lat - bounds[1].lat) /
                (self.bounds[0].lat - self.bounds[1].lat) * canvas_height as f64,
        };

        CropResult {
            image,
            cropped_bounds: bounds_approx,
            accurate_bounds: bounds,
            on_canvas_pos: [on_canvas_start, on_canvas_end],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::Distance;
    use crate::radar::radar_data::APILegends;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_crop() {
        let bounds = [
            Coordinate { lat: 0., lon: 0. },
            Coordinate { lat: -1., lon: 1. },
        ];

        let radar_bounds = [
            Coordinate {
                lat: 0.6,
                lon: -0.7,
            },
            Coordinate {
                lat: -1.4,
                lon: 1.3,
            },
        ];

        let radar_bounds2 = [
            Coordinate {
                lat: -0.1,
                lon: 0.1,
            },
            Coordinate {
                lat: -0.2,
                lon: 0.3,
            },
        ];

        let mut radar_test = RadarData {
            bounds: radar_bounds,
            city: "".to_string(),
            station: "".to_string(),
            code: "".to_string(),
            center: Coordinate {
                lat: -0.4,
                lon: 0.3,
            },
            range: Distance::Degrees(2.),
            priority: 0,
            images: vec![],
            legends: APILegends {
                levels: vec!(),
                colors: vec!(),
            },
        };

        let image = RgbaImage::new(2000, 2000);
        let im = RadarImagery::builder(bounds).build();

        let crop = im.crop(&image, &radar_test, 1827, 1827);

        // since crop is larger than what the bound SHOULD BE
        // the bounds may be larger than predicted, so it's fine

        assert_eq!(crop.image.height(), 1000);
        assert_eq!(crop.image.width(), 1000);
        assert_abs_diff_eq!(crop.cropped_bounds[0].lat, 0.0, epsilon = 0.0001);
        assert_abs_diff_eq!(crop.cropped_bounds[0].lon, 0.0, epsilon = 0.0001);
        assert_abs_diff_eq!(crop.cropped_bounds[1].lat, -1.0, epsilon = 0.0001);
        assert_abs_diff_eq!(crop.cropped_bounds[1].lon, 1.0, epsilon = 0.0001);

        assert_abs_diff_eq!(crop.on_canvas_pos[0].x, 0., epsilon = 0.0001);
        assert_abs_diff_eq!(crop.on_canvas_pos[0].y, 0., epsilon = 0.0001);
        assert_abs_diff_eq!(crop.on_canvas_pos[1].x, 1827., epsilon = 0.0001);
        assert_abs_diff_eq!(crop.on_canvas_pos[1].y, 1827., epsilon = 0.0001);

        radar_test.bounds = radar_bounds2;
        let crop = im.crop(&image, &radar_test, 1500, 1500);

        assert_eq!(crop.image.height(), 2000);
        assert_eq!(crop.image.width(), 2000);
        assert_abs_diff_eq!(crop.cropped_bounds[0].lat, -0.1, epsilon = 0.0001);
        assert_abs_diff_eq!(crop.cropped_bounds[0].lon, 0.1, epsilon = 0.0001);
        assert_abs_diff_eq!(crop.cropped_bounds[1].lat, -0.2, epsilon = 0.0001);
        assert_abs_diff_eq!(crop.cropped_bounds[1].lon, 0.3, epsilon = 0.0001);

        assert_abs_diff_eq!(crop.on_canvas_pos[0].x, 150., epsilon = 0.0001);
        assert_abs_diff_eq!(crop.on_canvas_pos[0].y, 150., epsilon = 0.0001);
        assert_abs_diff_eq!(crop.on_canvas_pos[1].x, 450., epsilon = 0.0001);
        assert_abs_diff_eq!(crop.on_canvas_pos[1].y, 300., epsilon = 0.0001);

        let radar_bounds3 = [
            Coordinate {
                lat: -0.45,
                lon: 0.46,
            },
            Coordinate {
                lat: -1.5,
                lon: 1.5,
            },
        ];

        radar_test.bounds = radar_bounds3;
        let crop = im.crop(&image, &radar_test, 1425, 1425);
        assert_eq!(crop.image.height(), 1048);
        assert_eq!(crop.image.width(), 1039);
        assert_abs_diff_eq!(crop.cropped_bounds[0].lat, -0.45, epsilon = 0.0001);
        assert_abs_diff_eq!(crop.cropped_bounds[0].lon, 0.46, epsilon = 0.0001);
        assert_abs_diff_eq!(crop.cropped_bounds[1].lat, -1.0002, epsilon = 0.0001);
        assert_abs_diff_eq!(crop.cropped_bounds[1].lon, 1.0002, epsilon = 0.0001);

        assert_abs_diff_eq!(crop.on_canvas_pos[0].x, 655.5, epsilon = 0.0001);
        assert_abs_diff_eq!(crop.on_canvas_pos[0].y, 641.25, epsilon = 0.0001);
        assert_abs_diff_eq!(crop.on_canvas_pos[1].x, 1425., epsilon = 0.0001);
        assert_abs_diff_eq!(crop.on_canvas_pos[1].y, 1425., epsilon = 0.0001);

        let crop = im.crop(&image, &radar_test, 1426, 1426);
        assert_eq!(crop.image.height(), 1048);
        assert_eq!(crop.image.width(), 1039);
        assert_abs_diff_eq!(crop.cropped_bounds[0].lat, -0.45, epsilon = 0.0001);
        assert_abs_diff_eq!(crop.cropped_bounds[0].lon, 0.46, epsilon = 0.0001);
        assert_abs_diff_eq!(crop.cropped_bounds[1].lat, -1.0002, epsilon = 0.0001);
        assert_abs_diff_eq!(crop.cropped_bounds[1].lon, 1.0002, epsilon = 0.0001);

        assert_abs_diff_eq!(crop.on_canvas_pos[0].x, 655.96, epsilon = 0.0001);
        assert_abs_diff_eq!(crop.on_canvas_pos[0].y, 641.7, epsilon = 0.0001);
        assert_abs_diff_eq!(crop.on_canvas_pos[1].x, 1426., epsilon = 0.0001);
        assert_abs_diff_eq!(crop.on_canvas_pos[1].y, 1426., epsilon = 0.0001);
    }
}
