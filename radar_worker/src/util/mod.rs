use image::{Rgba, RgbaImage};
use std::error::Error;
use url::Url;

pub fn get_domain(url: &str) -> String {
    match Url::parse(url) {
        Ok(url_obj) => match url_obj.domain() {
            Some(domain) => domain.to_string(),
            None => "[unknown domain]".to_string(),
        },
        Err(_) => "[invalid URL]".to_string(),
    }
}

pub fn gen_connection_err(e: reqwest::Error) -> Box<dyn Error + Send + Sync> {
    let url = match e.url() {
        Some(url) => url.to_string(),
        None => "".to_string(),
    };
    let domain = get_domain(&url);
    format!("Failed to connect to {}: {}", domain, e.to_string()).into()
}

pub fn overlay_image(a: RgbaImage, b: RgbaImage, opacity: f32) -> RgbaImage {
    assert!((0.0..=1.0).contains(&opacity));
    assert_eq!(a.width(), b.width());
    assert_eq!(a.height(), b.height());
    let mut a = a;

    let o_scale = opacity / 255f32;

    for y in 0..a.height() {
        for x in 0..a.width() {
            let a_pixel = a.get_pixel(x, y).0;
            let b_pixel = b.get_pixel(x, y).0;

            // originally this formula
            // modified to improve performance
            // because the opacity of the radar could only be 255 or 0
            // but still account for when it's not
            // let o = b_pixel[3] as f32 * o_scale;
            let o = match b_pixel[3] {
                255 => opacity,
                0 => 0.,
                _ => b_pixel[3] as f32 * o_scale,
            };

            // not rounding will improve performance
            // by an insane amount which is wild
            // so the 'common' way is to do a * (1 - opacity) + b * opacity
            // however, this formula works exactly the same
            // with one less multiplication, but one more addition
            // since addition is cheaper in performance, that's what we'll go for

            // most of the pixels will be zero when the radar is empty
            // in that case, why bother calculating at all?
            // even calculating by 0.0 takes time

            let (r, g, b) = match b_pixel[3] {
                0 => (a_pixel[0], a_pixel[1], a_pixel[2]),
                _ => {
                    let r = a_pixel[0] as i16 + ((b_pixel[0] as i16 - a_pixel[0] as i16) as f32 * o)
                        as i16;
                    let g = a_pixel[1] as i16 + ((b_pixel[1] as i16 - a_pixel[1] as i16) as f32 * o)
                        as i16;
                    let b = a_pixel[2] as i16 + ((b_pixel[2] as i16 - a_pixel[2] as i16) as f32 * o)
                        as i16;

                    (r as u8, g as u8, b as u8)
                }
            };

            let new_px = Rgba([r, g, b, a_pixel[3]]);
            a.put_pixel(x, y, new_px);
        }
    }

    a
}