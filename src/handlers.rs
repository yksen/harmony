use serenity::async_trait;
use songbird::{Event, EventContext, EventHandler as VoiceEventHandler, Songbird};
use std::sync::Arc;

pub struct TrackEndNotifier {
    pub manager: Arc<Songbird>,
    pub guild_id: serenity::model::id::GuildId,
}

#[async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        if let Some(handler_lock) = self.manager.get(self.guild_id) {
            let mut handler = handler_lock.lock().await;
            if handler.queue().is_empty() {
                handler.leave().await.ok();
            }
        }
        None
    }
}
