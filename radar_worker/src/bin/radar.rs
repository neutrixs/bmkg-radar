use clap::Parser;
use radar_worker::map::bounding::bounding_box;
use radar_worker::map::{MapImagery, MapStyle};
use radar_worker::radar::RadarImagery;
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
    max_tiles: i32
}

fn main() {
    let args = Args::parse();

    let _ = Runtime::new().unwrap().block_on(async {
        let bounds = bounding_box(args.place, Duration::from_secs(10)).await;
        if let Err(e) = bounds {
            panic!("{}", e);
        }
        let bounds = bounds.unwrap();

        let map = MapImagery::builder(bounds).map_style(MapStyle::Transport).build();
        let map_image = map.render().await;
        if let Err(e) = map_image {
            panic!("{}", e);
        }
        let map_image = map_image.unwrap();
        let width = map_image.width();
        let height = map_image.height();

        let im = RadarImagery::builder(bounds).build();
        let radar_render = im.render(width, height).await;
        if let Err(e) = radar_render {
            panic!("{}", e);
        }
        let radar_render = radar_render.unwrap();
        let radar_image = radar_render.image;

        let radar_image = overlay_image(map_image, radar_image, 0.7);

        radar_image.save(&args.output).unwrap();
    });
}