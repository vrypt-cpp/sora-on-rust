mod commands;
mod handler;
mod client;
mod config;

use chrono::Local;
use log::info;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            use std::io::Write;
            writeln!(buf, "{} [{}] - {}", Local::now().format("%H:%M:%S"), record.level(), record.args())
        })
        .init();
        let config = Arc::new(config::AppConfig::load()?);
    let mut bot = client::create_bot(Arc::clone(&config)).await?;
    info!("Starting Bot...");
    bot.run().await?.await?;
    Ok(())
}