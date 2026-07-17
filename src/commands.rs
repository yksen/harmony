use crate::{handlers, Context, Data, Error};
use songbird::{
    input::{Compose, Input, YoutubeDl},
    tracks::Track,
    Event, Songbird,
};
use std::sync::Arc;

pub struct TrackData {
    pub title: String,
    pub source: YoutubeDl<'static>,
}

pub(crate) fn all() -> Vec<poise::Command<Data, Error>> {
    vec![ping(), play(), skip(), now_playing(), queue(), loop_queue()]
}

fn fallback_title() -> String {
    "<UNKNOWN>".to_string()
}

async fn get_manager(ctx: &Context<'_>) -> Arc<Songbird> {
    songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client has not been initialized")
        .clone()
}

async fn try_join(ctx: &Context<'_>) -> Result<(), Error> {
    let (guild_id, author_channel) = {
        let guild = ctx.guild().unwrap();
        let channel = guild
            .voice_states
            .get(&ctx.author().id)
            .and_then(|voice_state| voice_state.channel_id);
        (guild.id, channel)
    };

    let manager = get_manager(ctx).await;
    let in_call = manager.get(guild_id).is_some();

    if author_channel.is_none() {
        if !in_call {
            ctx.say("You are not in a voice channel").await?;
        }
        return Ok(());
    }

    if let Ok(handler_lock) = manager.join(guild_id, author_channel.unwrap()).await {
        if !in_call {
            ctx.data()
                .guild_data
                .lock()
                .unwrap()
                .entry(guild_id)
                .or_default();
            let mut handler = handler_lock.lock().await;
            handler.add_global_event(
                Event::Track(songbird::TrackEvent::End),
                handlers::TrackEndNotifier {
                    manager: manager.clone(),
                    guild_id,
                    guild_data: ctx.data().guild_data.clone(),
                },
            );
        }
    } else {
        ctx.say("Failed to join the call").await?;
    }

    Ok(())
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

    let manager = get_manager(&ctx).await;
    let client = ctx.data().http_client.clone();

    let mut source = YoutubeDl::new(client, query);

    match source.aux_metadata().await {
        Ok(metadata) => {
            try_join(&ctx).await?;

            if let Some(handler_lock) = manager.get(ctx.guild_id().unwrap()) {
                let mut handler = handler_lock.lock().await;
                let title = metadata.title.unwrap_or(fallback_title());
                let data = Arc::new(TrackData {
                    title: title.clone(),
                    source: source.clone(),
                });
                let track = Track::new_with_data(Input::from(source), data);
                handler.enqueue(track).await;
                ctx.say(format!("Queued **{title}**")).await?;
            }
        }
        Err(why) => {
            ctx.say(format!(
                "Failed to get metadata of the song: `{}`",
                why.to_string().trim()
            ))
            .await?;
        }
    }

    Ok(())
}

/// Skip the current song
#[poise::command(slash_command, prefix_command, guild_only)]
async fn skip(ctx: Context<'_>) -> Result<(), Error> {
    let manager = get_manager(&ctx).await;

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
        ctx.say("Not in a call").await?;
    }

    Ok(())
}

/// Show the currently playing song
#[poise::command(slash_command, prefix_command, guild_only, rename = "now-playing")]
async fn now_playing(ctx: Context<'_>) -> Result<(), Error> {
    let manager = get_manager(&ctx).await;

    let guild_id = ctx.guild_id().unwrap();
    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        if let Some(track) = queue.current() {
            let title = track.data::<TrackData>().title.clone();
            ctx.say(format!("Now playing **{title}**")).await?;
        } else {
            ctx.say("Nothing is playing").await?;
        }
    } else {
        ctx.say("Not in a call").await?;
    }

    Ok(())
}

/// Show the current queue
#[poise::command(slash_command, prefix_command, guild_only)]
async fn queue(ctx: Context<'_>) -> Result<(), Error> {
    let manager = get_manager(&ctx).await;

    let guild_id = ctx.guild_id().unwrap();
    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();

        let mut message = "Queue is empty".to_string();
        for (index, track) in queue.current_queue().iter().enumerate() {
            let title = track.data::<TrackData>().title.clone();
            if index == 0 {
                message = format!("Now playing **{title}**\n");
            } else {
                message.push_str(&format!("{index}. **{title}**\n"));
            }
        }

        ctx.say(message).await?;
    } else {
        ctx.say("Not in a call").await?;
    }

    Ok(())
}

/// Loop the current queue
#[poise::command(slash_command, prefix_command, guild_only, rename = "loop")]
async fn loop_queue(ctx: Context<'_>) -> Result<(), Error> {
    let manager = get_manager(&ctx).await;

    let guild_id = ctx.guild_id().unwrap();
    if manager.get(guild_id).is_some() {
        let loop_state = {
            let mut data = ctx.data().guild_data.lock().unwrap();
            data.entry(guild_id)
                .and_modify(|data| data.loop_queue = !data.loop_queue);
            data[&guild_id].loop_queue
        };

        if loop_state {
            ctx.say("Looping the queue").await?;
        } else {
            ctx.say("Stopped looping the queue").await?;
        }
    } else {
        ctx.say("Not in a call").await?;
    }

    Ok(())
}
