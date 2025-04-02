use image::{ImageFormat::Png, RgbaImage};
use radar_worker::radar::RadarImagery;
use serenity::prelude::TypeMapKey;
use std::io::Cursor;
use std::sync::Arc;
use tokio::sync::Mutex;

pub(crate) type Error = Box<dyn std::error::Error + Send + Sync>;
pub(crate) struct Data {
    pub radar: Arc<Mutex<RadarImagery>>,
}
impl TypeMapKey for Data {
    type Value = Arc<Mutex<RadarImagery>>;
}

pub(crate) type Context<'a> = poise::Context<'a, Data, Error>;

pub(crate) fn rgba_image_to_bytes(image: RgbaImage) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut cursor = Cursor::new(&mut buf);
    image.write_to(&mut cursor, Png).unwrap();
    buf
}