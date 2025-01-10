use serenity::builder::CreateCommand;
use serenity::model::application::ResolvedOption;

pub fn run(_options: &[ResolvedOption]) -> String {
    "Play".to_string()
}

pub fn register() -> CreateCommand {
    CreateCommand::new("play").description("Play a song")
}