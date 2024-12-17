use clap::Parser;
use radar_worker::map::bounding::bounding_box;
use radar_worker::map::{MapImagery, ZoomSetting};
use std::path::PathBuf;
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
        let bounds = bounding_box(args.place).await.unwrap();
        let imagery = MapImagery::builder(bounds)
            .zoom_setting(ZoomSetting::MaxTiles(args.max_tiles))
            .build();
        let image = imagery.render().await;
        if let Err(e) = image {
            panic!("{}", e);
        }
        let image = image.unwrap();
        let path = PathBuf::from(&args.output);

        image.save(&path).expect("Failed to save the image");
    });
}