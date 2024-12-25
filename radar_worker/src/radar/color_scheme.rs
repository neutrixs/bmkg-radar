use crate::radar::RadarData;
use image::Rgba;
use std::num::ParseIntError;

const COLOR_SCHEME: [Rgba<u8>; 14] = [
    Rgba([173, 216, 230, 255]), // Light Blue
    Rgba([0, 0, 255, 255]),     // Medium Blue
    Rgba([0, 0, 139, 255]),     // Dark Blue
    Rgba([0, 255, 0, 255]),     // Green
    Rgba([50, 205, 50, 255]),   // Lime Green
    Rgba([255, 255, 0, 255]),   // Yellow
    Rgba([255, 215, 0, 255]),   // Gold
    Rgba([255, 165, 0, 255]),   // Orange
    Rgba([255, 140, 0, 255]),   // Dark Orange
    Rgba([255, 0, 0, 255]),     // Red
    Rgba([139, 0, 0, 255]),     // Dark Red
    Rgba([255, 0, 255, 255]),   // Magenta
    Rgba([128, 0, 128, 255]),   // Purple
    Rgba([0, 0, 0, 255]),       // Black
];

pub(crate) fn hex_to_rgb(hex: &str) -> Result<Rgba<u8>, ParseIntError> {
    let hex = hex.trim_start_matches('#');

    let r = u8::from_str_radix(&hex[0..2], 16)?;
    let g = u8::from_str_radix(&hex[2..4], 16)?;
    let b = u8::from_str_radix(&hex[4..6], 16)?;

    Ok(Rgba([r, g, b, 255]))
}


pub(crate) fn pixel_to_color_scheme(pixel: Rgba<u8>, radar: &RadarData) -> Rgba<u8> {
    let default_scheme_iter = radar.legends.colors.iter().enumerate();
    let mut pixel = pixel;

    for (i, val) in default_scheme_iter {
        if val != &pixel {
            continue;
        }

        if i >= 14 {
            pixel = Rgba([0, 0, 0, 0]);
        } else {
            pixel = COLOR_SCHEME[i];
        }
    }

    pixel
}