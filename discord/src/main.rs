mod commands;
mod common;

use crate::commands::get_image::get_image;
use common::*;
use serenity::all::GatewayIntents;
use std::env;

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Invalid / non-existent DISCORD_TOKEN env var");
    let _ = env::var("BMKG_APIKEY").expect("Invalid / non-existent BMKG_APIKEY env var");
    let _ = env::var("THUNDERFOREST_APIKEY")
        .expect("Invalid / non-existent THUNDERFOREST_APIKEY env var");
    let intents = GatewayIntents::empty();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![get_image()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::Client::builder(&token, intents)
        .framework(framework)
        .await;
    client.unwrap().start_autosharded().await.unwrap();
}
