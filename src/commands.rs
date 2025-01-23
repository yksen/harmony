use crate::{handlers, Context, Data, Error};
use serenity::prelude::TypeMapKey;
use songbird::{input::Compose, Event};

pub fn all() -> Vec<poise::Command<Data, Error>> {
    vec![ping(), play(), skip(), now_playing(), queue()]
}

/// Ping command
#[poise::command(slash_command, prefix_command)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Pong").await?;
    Ok(())
}

/// Play a song
#[poise::command(slash_command, prefix_command, guild_only)]
async fn play(ctx: Context<'_>, #[description = "YouTube URL"] query: String) -> Result<(), Error> {
    ctx.defer().await?;

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
                handlers::TrackEndNotifier {
                    manager: manager.clone(),
                    guild_id,
                },
            );
        }
    } else {
        ctx.say("You are not in a voice channel").await?;
    }

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        let client = ctx.data().http_client.clone();
        let mut source = songbird::input::YoutubeDl::new(client, query);

        let input = songbird::input::Input::from(source.clone());
        let handle = handler.enqueue_input(input).await;

        // TODO: Faster way to get metadata
        if let Ok(metadata) = source.aux_metadata().await {
            let title = metadata.title.unwrap_or(fallback_title());

            let mut typemap = handle.typemap().write().await;
            typemap.insert::<SongTitle>(title.clone());

            ctx.say(format!("Queued **{title}**")).await?;
        } else {
            ctx.say("Song queued").await?;
        }
    }

    Ok(())
}

/// Skip the current song
#[poise::command(slash_command, prefix_command, guild_only)]
async fn skip(ctx: Context<'_>) -> Result<(), Error> {
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

struct SongTitle;

impl TypeMapKey for SongTitle {
    type Value = String;
}

/// Show the currently playing song
#[poise::command(slash_command, prefix_command, guild_only, rename = "now-playing")]
async fn now_playing(ctx: Context<'_>) -> Result<(), Error> {
    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client has not been initialized")
        .clone();

    let guild_id = ctx.guild_id().unwrap();
    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        if let Some(track) = queue.current() {
            let typemap = track.typemap().read().await;
            let title = typemap
                .get::<SongTitle>()
                .cloned()
                .unwrap_or_else(fallback_title);
            ctx.say(format!("Now playing **{title}**")).await?;
        } else {
            ctx.say("Nothing is playing").await?;
        }
    } else {
        ctx.say("Not in a voice channel").await?;
    }

    Ok(())
}

fn fallback_title() -> String {
    "<UNKNOWN>".to_string()
}

/// Show the current queue
#[poise::command(slash_command, prefix_command, guild_only)]
async fn queue(ctx: Context<'_>) -> Result<(), Error> {
    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client has not been initialized")
        .clone();

    let guild_id = ctx.guild_id().unwrap();
    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();

        let mut message = "Queue is empty".to_string();
        for (index, track) in queue.current_queue().iter().enumerate() {
            let typemap = track.typemap().read().await;
            let title = typemap
                .get::<SongTitle>()
                .cloned()
                .unwrap_or_else(fallback_title);
            if index == 0 {
                message = format!("Now playing **{title}**\n");
            } else {
                message.push_str(&format!("{index}. {title}\n"));
            }
        }

        ctx.say(message).await?;
    } else {
        ctx.say("Not in a voice channel").await?;
    }

    Ok(())
}
