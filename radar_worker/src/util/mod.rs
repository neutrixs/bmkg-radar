use image::{Rgba, RgbaImage};

pub fn overlay_image(a: RgbaImage, b: RgbaImage, opacity: f32) -> RgbaImage {
    assert!((0.0..=1.0).contains(&opacity));
    assert_eq!(a.width(), b.width());
    assert_eq!(a.height(), b.height());
    let mut a = a;

    let o_scale = opacity / 255f32;
    let opacity_inv = 1. - opacity;

    for y in 0..a.height() {
        for x in 0..a.width() {
            let a_pixel = a.get_pixel(x, y).0;
            let b_pixel = b.get_pixel(x, y).0;

            // originally this formula
            // modified to improve performance
            // because the opacity of the radar could only be 255 or 0
            // but still account for when it's not
            // let o = b_pixel[3] as f32 * o_scale;
            // let inv_o = 1.0 - o;
            let (o, inv_o) = match b_pixel[3] {
                255 => (opacity, opacity_inv),
                0 => (0., 1.),
                _ => {
                    let o = b_pixel[3] as f32 * o_scale;
                    let inv_o = 1.0 - o;
                    (o, inv_o)
                }
            };

            // not rounding will improve performance
            // by around 3.8x.. which is wild
            let r = (a_pixel[0] as f32 * inv_o + b_pixel[0] as f32 * o) as u8;
            let g = (a_pixel[1] as f32 * inv_o + b_pixel[1] as f32 * o) as u8;
            let b = (a_pixel[2] as f32 * inv_o + b_pixel[2] as f32 * o) as u8;

            let new_px = Rgba([r, g, b, a_pixel[3]]);
            a.put_pixel(x, y, new_px);
        }
    }

    a
}