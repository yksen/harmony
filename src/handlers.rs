use serenity::{all::GuildId, async_trait};
use songbird::{Event, EventContext, EventHandler as VoiceEventHandler, Songbird};
use std::sync::Arc;
use tracing::error;

pub struct TrackEndNotifier {
    pub manager: Arc<Songbird>,
    pub guild_id: GuildId,
}

#[async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        let Some(handler_lock) = self.manager.get(self.guild_id) else {
            return None;
        };

        let should_leave = {
            let handler = handler_lock.lock().await;
            handler.queue().is_empty()
        };

        if should_leave {
            if let Err(why) = self.manager.remove(self.guild_id).await {
                error!("Failed to remove handler: {why}");
            }
        }

        None
    }
}
