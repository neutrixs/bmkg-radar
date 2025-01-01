use image::{ImageFormat::Png, RgbaImage};
use std::io::Cursor;


pub(crate) type Error = Box<dyn std::error::Error + Send + Sync>;
pub(crate) struct Data {}
pub(crate) type Context<'a> = poise::Context<'a, crate::Data, crate::Error>;

pub(crate) fn rgba_image_to_bytes(image: RgbaImage) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut cursor = Cursor::new(&mut buf);
    image.write_to(&mut cursor, Png).unwrap();
    buf
}