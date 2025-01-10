mod commands;

use log::{error, info};
use serenity::all::{
    Command, CreateInteractionResponse, CreateInteractionResponseMessage, Interaction,
};
use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::env;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
        for command in commands::all() {
            let res = Command::create_global_command(&ctx.http, command).await;
            info!("Registered command: {:?}", res);
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let content = match command.data.name.as_str() {
                "ping" => Some(commands::ping::run(&command.data.options())),
                "play" => Some(commands::play::run(&command.data.options())),
                _ => None,
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    error!("Cannot respond to command: {:?}", why);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    flexi_logger::init();
    dotenv::dotenv().expect("Failed to read .env file");

    let token = env::var("DISCORD_TOKEN").expect("Expected a `DISCORD_TOKEN` in the environment");
    let mut client = Client::builder(&token, GatewayIntents::empty())
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
