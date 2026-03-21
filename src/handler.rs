use log::{error, info};
use std::sync::Arc;
use crate::state::AppState;
use wacore::types::events::Event;
use whatsapp_rust::client::Client;
use crate::commands::cmd::COMMANDS;
use crate::config::AppConfig;
use crate::utils::MessageExt;
pub async fn event_handler(event: Event, client: Arc<Client>, config: Arc<AppConfig>, state: Arc<AppState>) {
    match event {
        Event::Connected(_) => {
            info!("✅ Bot connected successfully!");
        }
        Event::Message(msg, info) => {
            if let Some(exp) = msg.get_expiration_timer() {
                state.set_expiration(&info.source.chat.to_string(), exp);
            }
            if let Some(text) = msg.text_content() {
                let matched_prefix = config.prefixes.iter().find(|p| text.starts_with(*p));
             let prefix = match matched_prefix {
                Some(p) => p,
                None => return,
            };

            let msg_arc = Arc::from(msg);
            let info_arc = Arc::new(info);
                let body = &text[prefix.len()..];
                let args: Vec<&str> = body.split_whitespace().collect();
                if args.is_empty() { return; }
                let cmd_name = args[0].to_lowercase();
                for cmd in COMMANDS {
                    if cmd.name() == cmd_name || cmd.aliases().contains(&cmd_name.as_str()) {
                        let ctx = crate::commands::cmd::Context {
                            client: Arc::clone(&client),
                            msg: Arc::clone(&msg_arc),  
                            info: Arc::clone(&info_arc),
                            state: Arc::clone(&state),
                        };
                
                        if let Err(e) = cmd.execute(ctx).await {
                            error!("Error executing {}: {}", cmd_name, e);
                        }
                        return;
                    }
                }
            }
        }
        _ => {}
    }
}