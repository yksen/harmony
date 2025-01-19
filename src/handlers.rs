use serenity::async_trait;
use songbird::{Event, EventContext, EventHandler as VoiceEventHandler};

pub struct TrackEndNotifier {}

#[async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        None
    }
}
