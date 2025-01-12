use log::{error, info};
use poise::serenity_prelude as serenity;
use songbird::SerenityInit;
use std::env;

struct Data {
    http_client: reqwest::Client,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Ping command
#[poise::command(slash_command, prefix_command)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Pong").await?;
    Ok(())
}

/// Play a song
#[poise::command(slash_command, prefix_command, guild_only)]
async fn play(ctx: Context<'_>, #[description = "YouTube URL"] query: String) -> Result<(), Error> {
    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client has not been initialized")
        .clone();

    let guild = ctx.guild_id().unwrap();
    let sender_channel = ctx
        .guild()
        .unwrap()
        .voice_states
        .get(&ctx.author().id)
        .and_then(|voice_state| voice_state.channel_id);

    if let Some(channel) = sender_channel {
        if let Ok(handler_lock) = manager.join(guild, channel).await {
            let mut handler = handler_lock.lock().await;
        }

        if let Some(handler_lock) = manager.get(guild) {
            let mut handler = handler_lock.lock().await;
            let source = songbird::input::YoutubeDl::new(ctx.data().http_client.clone(), query);
            let _ = handler.play_input(source.clone().into());
            ctx.say("Playing song").await?;
        }
    } else {
        ctx.say("Not in a voice channel").await?;
    }

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
            commands: vec![ping(), play()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    http_client: reqwest::Client::new(),
                })
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
