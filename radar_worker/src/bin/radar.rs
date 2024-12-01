use std::env;
use radar_worker::map::{MapImagery, ZoomSetting};
use radar_worker::map::bounding::bounding_box;
use tokio::runtime::Runtime;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    args = args.drain(1..).collect();

    let place = args.join(" ");
    println!("{}", place);

    let _ = Runtime::new().unwrap().block_on(async {
        let bounds = bounding_box(String::from("Bandung")).await.unwrap();
        let _ = MapImagery::builder(bounds)
            .zoom_setting(ZoomSetting::MaxTiles(200))
            .build();
    });
}