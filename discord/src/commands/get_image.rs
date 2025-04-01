use crate::common::{rgba_image_to_bytes, Context};
use image::RgbaImage;
use poise::CreateReply;
use radar_worker::map::{bounding, MapImagery, MapStyle};
use radar_worker::radar::{RadarData, RadarImagery, RenderResult};
use radar_worker::util::overlay_image;
use serenity::all::CreateAttachment;
use serenity::futures;
use std::time::Duration;

fn format_description(radars: Vec<RadarData>) -> String {
    if radars.len() == 0 {
        return String::from("No radars used.");
    }

    let mut format = match radars.len() {
        1 => String::from("Used radar:"),
        _ => String::from("Used radars:"),
    };

    let not_enough = String::from("\netc.");

    for radar in radars {
        let code = radar.code;
        let city = radar.city;
        let time = radar.images.last().unwrap().time.timestamp();

        let append = format!("\n{} ({}): <t:{}:R>", code, city, time);

        // discord character limit
        if format.len() + append.len() + not_enough.len() <= 2000 {
            format += &append;
        } else {
            format += &not_enough;
            break;
        }
    }

    format
}

async fn send_error_message(ctx: &Context<'_>, error: Box<dyn std::error::Error + Send + Sync>) {
    let message = format!("Error: {}", error.to_string());
    let _ = ctx.say(message).await;
}

/// get the radar imagery of a place
#[poise::command(slash_command)]
pub async fn get_image(
    ctx: Context<'_>,
    #[description = "Place to search for"] place: String,
    #[description = "Use the map's dark mode version"] dark_mode: Option<bool>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    ctx.defer().await?;

    let bounding_box = bounding::bounding_box(place, Duration::from_secs(10)).await;
    if let Err(e) = bounding_box {
        send_error_message(&ctx, e).await;
        return Ok(());
    }
    let bounding_box = bounding_box?;

    let style = match dark_mode {
        Some(false) => MapStyle::Transport,
        _ => MapStyle::TransportDark,
    };

    let imagery = MapImagery::builder(bounding_box)
        .map_style(style)
        .timeout_duration(Duration::from_secs(20))
        .build();
    let mut radar = RadarImagery::builder()
        .enforce_age_threshold(true)
        .timeout_duration(Duration::from_secs(20))
        .build();
    let (width, height) = imagery.get_image_size();

    let (map_result, radar_result) = futures::join!(imagery.render(), radar.render(width, height,
    bounding_box));
    if let Err(e) = map_result {
        send_error_message(&ctx, e).await;
        return Ok(());
    }
    if let Err(e) = radar_result {
        send_error_message(&ctx, e).await;
        return Ok(());
    }
    // explicit type because RustRover is buggy
    // it doesn't know how to deal with the macro
    let map_image: RgbaImage = map_result?;
    let radar_result: RenderResult = radar_result?;
    let radar_image = radar_result.image;

    let overlayed = overlay_image(map_image, radar_image, 0.5);
    let overlayed = rgba_image_to_bytes(overlayed);
    let attachment = CreateAttachment::bytes(overlayed, "radar.png");

    let _ = ctx
        .send(
            CreateReply::default()
                .content(format_description(radar_result.used_radars))
                .attachment(attachment),
        )
        .await;

    Ok(())
}
