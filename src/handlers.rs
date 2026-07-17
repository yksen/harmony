use crate::{commands::TrackData, GuildData};
use serenity::{all::GuildId, async_trait};
use songbird::{
    input::Input,
    tracks::{PlayMode, Track},
    Event, EventContext, EventHandler as VoiceEventHandler, Songbird,
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
            let data = track.data::<TrackData>();
            let song_ended = matches!(state.playing, PlayMode::End);
            let should_loop = {
                let mut guild_data = self.guild_data.lock().unwrap();
                guild_data.entry(self.guild_id).or_default().loop_queue && song_ended
            };

            if should_loop {
                let input = Input::from(data.source.clone());
                let track = Track::new_with_data(input, data.clone());
                handler_lock.lock().await.enqueue(track).await;
            } else if handler_lock.lock().await.queue().is_empty() {
                if let Err(why) = self.manager.remove(self.guild_id).await {
                    error!("Failed to remove handler: {why}");
                }
            }
        }

        None
    }
}
