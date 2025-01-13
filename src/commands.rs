use tracing::info;

use crate::{Context, Error};

/// Ping command
#[poise::command(slash_command, prefix_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Pong").await?;
    Ok(())
}

/// Play a song
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "YouTube URL"] query: String,
) -> Result<(), Error> {
    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client  has not been initialized")
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
            let _ = handler_lock.lock().await;
        }

        if let Some(handler_lock) = manager.get(guild) {
            let mut handler = handler_lock.lock().await;
            let client = ctx.data().http_client.clone();
            let source = songbird::input::YoutubeDl::new(client, query);

            let mut input = songbird::input::Input::from(source);
            let artist = input.aux_metadata().await?.artist.unwrap_or_default();
            let song_name = input.aux_metadata().await?.title.unwrap_or_default();

            let _ = handler.play_input(input);
            ctx.say(format!("Queued ***{artist} - {song_name}***"))
                .await?;
        }
    } else {
        ctx.say("Not in a voice channel").await?;
    }

    Ok(())
}
