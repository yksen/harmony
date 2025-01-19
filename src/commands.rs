use crate::{handlers, Context, Error};
use songbird::{input::Compose, Event};
use tracing::info;

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
    let (guild_id, channel_id) = {
        let guild = ctx.guild().unwrap();
        let channel_id = guild
            .voice_states
            .get(&ctx.author().id)
            .and_then(|voice_state| voice_state.channel_id);

        (guild.id, channel_id)
    };

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client has not been initialized")
        .clone();

    if let Some(channel) = channel_id {
        if let Ok(handler_lock) = manager.join(guild_id, channel).await {
            let mut handler = handler_lock.lock().await;
            handler.add_global_event(
                Event::Track(songbird::TrackEvent::End),
                handlers::TrackEndNotifier {},
            );
        }
    } else {
        ctx.say("You are not in a voice channel").await?;
    }

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        let client = ctx.data().http_client.clone();
        let source = songbird::input::YoutubeDl::new(client, query);

        // TODO: Print metadata
        ctx.say("Song queued").await?;

        let input = songbird::input::Input::from(source);
        handler.enqueue_input(input).await;
    }

    Ok(())
}

/// Skip the current song
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn skip(ctx: Context<'_>) -> Result<(), Error> {
    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client has not been initialized")
        .clone();

    let guild_id = ctx.guild_id().unwrap();
    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        if queue.is_empty() {
            ctx.say("Queue is empty").await?;
            return Ok(());
        }
        let _ = queue.skip();
        ctx.say("Skipped").await?;
    } else {
        ctx.say("Not in a voice channel").await?;
    }

    Ok(())
}
