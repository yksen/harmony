use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
pub struct Args {
    /// Check for updates
    #[arg(long, short)]
    pub update: bool,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap_or_default();

    let args = Args::parse();
    if args.update {
        match update() {
            Ok(status) => {
                println!("{status:#}");
            }
            Err(why) => {
                eprintln!("Update failed: {why:#}");
            }
        }
        return;
    }

    let token = std::env::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN in the environment");
    let result = harmony::run(token).await;

    if let Err(why) = result {
        eprintln!("An error occurred: {why:#}");
    }
}

fn update() -> Result<String> {
    let status = self_update::backends::github::Update::configure()
        .repo_owner("yksen")
        .repo_name("harmony")
        .bin_name("harmony")
        .show_download_progress(true)
        .current_version(env!("CARGO_PKG_VERSION"))
        .build()?
        .update()?;

    Ok(status.to_string())
}
