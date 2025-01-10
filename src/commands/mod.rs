use serenity::all::CreateCommand;

pub mod ping;
pub mod play;

pub fn all() -> Vec<CreateCommand> {
    vec![ping::register(), play::register()]
}
