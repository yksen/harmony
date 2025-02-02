mod commands;
mod handlers;

use clap::Parser;
use poise::serenity_prelude as serenity;
use serenity::all::GuildId;
use songbird::SerenityInit;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tracing::{error, info};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Debug)]
struct GuildData {
    loop_queue: bool,
}

impl Default for GuildData {
    fn default() -> Self {
        Self { loop_queue: false }
    }
}

struct Data {
    http_client: reqwest::Client,
    guild_data: Arc<Mutex<HashMap<GuildId, GuildData>>>,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            http_client: reqwest::Client::new(),
            guild_data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Check for updates
    #[arg(long, short)]
    update: bool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().unwrap_or_default();

    let args = Args::parse();
    if args.update {
        if let Err(why) = update() {
            println!("\nUpdate failed: {why}");
        }
        return;
    }

    let token = std::env::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN in the environment");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::all(),
            on_error: |error| Box::pin(on_error(error)),
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data::default())
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .register_songbird()
        .await;

    if let Err(why) = client.unwrap().start().await {
        error!("Client error: {:?}", why);
    }
}

fn update() -> anyhow::Result<()> {
    let status = self_update::backends::github::Update::configure()
        .repo_owner("yksen")
        .repo_name("harmony")
        .bin_name("harmony")
        .show_download_progress(true)
        .current_version(env!("CARGO_PKG_VERSION"))
        .build()?
        .update()?;

    println!("\nUpdate successful: {status}");
    Ok(())
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            error!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                error!("Error while handling error: {}", e)
            }
        }
    }
}
