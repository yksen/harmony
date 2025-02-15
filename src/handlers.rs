use crate::{
    commands::{SongSource, SongTitle},
    GuildData,
};
use serenity::{all::GuildId, async_trait};
use songbird::{
    tracks::PlayMode, Event, EventContext, EventHandler as VoiceEventHandler, Songbird,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tracing::{error, info};

pub struct TrackEndNotifier {
    pub manager: Arc<Songbird>,
    pub guild_id: GuildId,
    pub(crate) guild_data: Arc<Mutex<HashMap<GuildId, GuildData>>>,
}

#[async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        info!("Track ended in guild {}", self.guild_id);

        let handler_lock = self.manager.get(self.guild_id)?;

        if let EventContext::Track(&[(state, track)]) = ctx {
            let type_map = track.typemap().read().await;
            let source = type_map.get::<SongSource>().cloned().unwrap();
            let input = songbird::input::Input::from(source.clone());
            let song_ended = matches!(state.playing, PlayMode::End);
            let should_loop = {
                let mut data = self.guild_data.lock().unwrap();
                data.entry(self.guild_id).or_default().loop_queue && song_ended
            };

            if should_loop {
                let track_handle = handler_lock.lock().await.enqueue_input(input).await;
                let title = type_map.get::<SongTitle>().cloned().unwrap();
                let mut type_map = track_handle.typemap().write().await;
                type_map.insert::<SongTitle>(title);
                type_map.insert::<SongSource>(source);
            } else if handler_lock.lock().await.queue().is_empty() {
                if let Err(why) = self.manager.remove(self.guild_id).await {
                    error!("Failed to remove handler: {why}");
                }
            }
        }

        None
    }
}
