use clap::Parser;
use image::RgbaImage;
use radar_worker::map::bounding::bounding_box;
use radar_worker::map::{MapImagery, MapStyle};
use radar_worker::radar::{RadarImagery, RenderResult};
use radar_worker::util::overlay_image;
use std::time::Duration;
use tokio::runtime::Runtime;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, help = "Specify the place name to render")]
    place: String,
    #[arg(
        short,
        long,
        value_name = "PATH",
        help = "Specify the output path",
        default_value = "output.png"
    )]
    output: String,
    #[arg(
        long,
        value_name = "AMOUNT",
        help = "Specify the maximum amount of map tiles",
        default_value = "50"
    )]
    max_tiles: i32,
}

fn main() {
    let args = Args::parse();

    let _ = Runtime::new().unwrap().block_on(async {
        let bounds = bounding_box(args.place, Duration::from_secs(10)).await;
        if let Err(e) = bounds {
            panic!("{}", e);
        }
        let bounds = bounds.unwrap();

        let map = MapImagery::builder(bounds)
            .map_style(MapStyle::Transport)
            .build();
        let radar = RadarImagery::builder(bounds)
            .enforce_age_threshold(true)
            .build();
        let (width, height) = map.get_image_size();

        let (map_result, radar_result) = futures::join!(map.render(), radar.render(width, height));
        if let Err(e) = map_result {
            panic!("{}", e);
        }
        if let Err(e) = radar_result {
            panic!("{}", e);
        }

        let map_image: RgbaImage = map_result.unwrap();
        let radar_result: RenderResult = radar_result.unwrap();
        let radar_image = radar_result.image;

        let radar_image = overlay_image(map_image, radar_image, 0.7);

        radar_image.save(&args.output).unwrap();
    });
}
