use log::{error, info};
use poise::serenity_prelude as serenity;
use songbird::SerenityInit;
use std::env;

struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command, prefix_command)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Pong").await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    flexi_logger::init();
    dotenv::dotenv().expect("Failed to read .env file");

    let token = env::var("DISCORD_TOKEN").expect("Expected a `DISCORD_TOKEN` in the environment");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
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
