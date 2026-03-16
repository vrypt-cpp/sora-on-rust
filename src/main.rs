use chrono::Local;
use log::{error, info};
use wacore::pair_code::{PairCodeOptions, PlatformId};
use std::sync::Arc;
use wacore::proto_helpers::MessageExt;
use wacore::types::events::Event;
use waproto::whatsapp as wa;
use whatsapp_rust::bot::Bot;
use whatsapp_rust::store::SqliteStore;
use whatsapp_rust_tokio_transport::TokioWebSocketTransportFactory;
use whatsapp_rust_ureq_http_client::UreqHttpClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            use std::io::Write;
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .init();

    // Initialize backend
    let backend = Arc::new(SqliteStore::new("whatsapp.db").await?);
    info!("SQLite backend initialized");

    // Build bot
    let mut bot = Bot::builder()
        .with_backend(backend)
        .with_transport_factory(TokioWebSocketTransportFactory::new())
        .with_http_client(UreqHttpClient::new())
        .with_pair_code(PairCodeOptions{
            phone_number: "62895404956278".to_string(),
            show_push_notification: true,
            custom_code: Some("HELLSTAR".to_string()),
            platform_id: PlatformId::Chrome,
            platform_display: "Chrome (Linux)".to_string(),
        })
        .on_event(|event, client| async move {
            match event {
                Event::PairingCode { code, .. } => {
                    info!("Pair with this code: {}", code);
                }
                Event::Message(msg, info) => {
                    if let Some(text) = msg.text_content() {
                        info!("Received: {} from {}", text, info.source.sender);

                        if text == "{ping" {
                            let reply = wa::Message {
                                conversation: Some("pong".to_string()),
                                ..Default::default()
                            };

                            if let Err(e) = client.send_message(info.source.chat, reply).await {
                                error!("Failed to send reply: {}", e);
                            }
                        }
                    }
                }
                Event::Connected(_) => {
                    info!("✅ Bot connected successfully!");
                }
                Event::LoggedOut(_) => {
                    error!("❌ Bot was logged out");
                }
                _ => {}
            }
        })
        .build()
        .await?;

    info!("Starting bot...");
    bot.run().await?.await?;
    Ok(())
}