use crate::common::{rgba_image_to_bytes, Context, Error};
use poise::CreateReply;
use radar_worker::map::{bounding, MapImagery, MapStyle};
use radar_worker::radar::{RadarData, RadarImagery};
use radar_worker::util::overlay_image;
use serenity::all::CreateAttachment;
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

/// get the radar imagery of a place
#[poise::command(slash_command)]
pub async fn get_image(
    ctx: Context<'_>,
    #[description = "Place to search for"] place: String,
    #[description = "Use the map's dark mode version"] dark_mode: Option<bool>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let bounding_box = bounding::bounding_box(place, Duration::from_secs(10)).await;
    if let Err(e) = bounding_box {
        let message = format!("Error: {}", e.to_string());
        let _ = ctx.say(message).await;
        return Ok(());
    }
    let bounding_box = bounding_box.unwrap();

    let style = match dark_mode {
        Some(true) => MapStyle::TransportDark,
        _ => MapStyle::Transport,
    };

    let imagery = MapImagery::builder(bounding_box)
        .map_style(style)
        .timeout_duration(Duration::from_secs(20))
        .build();
    let imagery_render_result = imagery.render().await;
    if let Err(e) = imagery_render_result {
        let message = format!("Error: {}", e.to_string());
        let _ = ctx.say(message).await;
        return Ok(());
    }

    let imagery_rendered = imagery_render_result.unwrap();
    let (width, height) = imagery_rendered.dimensions();

    let radar = RadarImagery::builder(bounding_box)
        .enforce_age_threshold(true)
        .timeout_duration(Duration::from_secs(20))
        .build();

    let radar_render_result = radar.render(width, height).await;
    if let Err(e) = radar_render_result {
        let message = format!("Error: {}", e.to_string());
        let _ = ctx.say(message).await;
        return Ok(());
    }
    let radar_render_result = radar_render_result.unwrap();
    let radar_rendered = radar_render_result.image;

    let overlayed = overlay_image(imagery_rendered, radar_rendered, 0.7);
    let overlayed = rgba_image_to_bytes(overlayed);

    let attachment = CreateAttachment::bytes(overlayed, "radar.png");

    let _ = ctx
        .send(
            CreateReply::default()
                .content(format_description(radar_render_result.used_radars))
                .attachment(attachment),
        )
        .await;

    Ok(())
}
