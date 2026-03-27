use log::{error, info};
use wacore::stanza::GroupNotificationAction;
use std::sync::Arc;
use crate::state::AppState;
use wacore::{client::context::SendContextResolver, types::events::Event};
use whatsapp_rust::client::Client;
use crate::commands::cmd::COMMANDS;
use crate::config::AppConfig;
use crate::utils::MessageExt;
use tokio::sync::RwLock;
use std::sync::LazyLock;
static SUPERUSER_LID: LazyLock<RwLock<Option<String>>> = LazyLock::new(|| RwLock::new(None));
pub async fn event_handler(event: Event, client: Arc<Client>, config: Arc<AppConfig>, state: Arc<AppState>) {
    match event {
        Event::Connected(_) => {
            info!("✅ Bot connected successfully!");
            if let Some(su_pn) = &config.superuser {
                if let Some(jid) = client.get_lid_for_phone(su_pn).await {
                    let lid_user = jid.to_string();
                    
                    let mut lock = SUPERUSER_LID.write().await;
                    *lock = Some(lid_user.clone());
                }
            }
            
        }
        Event::Message(msg, info) => {
            // println!("{:#?}", msg);
            let start = std::time::Instant::now();
            let msg_arc = Arc::new(*msg);
            let info_arc = Arc::new(info);
            if let Some(exp) = msg_arc.get_expiration_timer() {
                state.set_expiration(&info_arc.source.chat.to_string(), exp);
                println!("Expiration received: {}", exp);
            }
            
            if let Some(text) = msg_arc.text_content() {
            let matched_prefix = config.prefixes.iter().find(|p| text.starts_with(*p));
            let prefix = match matched_prefix {
                Some(p) => p,
                None => return,
            };
            if config.mode == "self" {
                let sender = &info_arc.source.sender.user;
    
                let me = info_arc.source.is_from_me;
                let su = if info_arc.source.sender.is_lid() {
                    if let Ok(lock) = SUPERUSER_LID.try_read() {
                        lock.as_deref() == Some(sender.as_str())
                    } else {
                        false
                    }
                } else {
                    config.superuser.as_ref() == Some(&sender)
                };
                let privileged = me || su;
                if !privileged {
                    return;
                }
            }
                let body = &text[prefix.len()..];
                let args: Vec<&str> = body.split_whitespace().collect();
                if args.is_empty() { return; }
                let cmd_name = args[0];
                for cmd in COMMANDS {
                    if cmd.name().eq_ignore_ascii_case(cmd_name) || cmd.aliases().iter().any(|&alias| alias.eq_ignore_ascii_case(cmd_name)) {
                        let ctx = crate::commands::cmd::Context {
                            client: Arc::clone(&client),
                            msg: Arc::clone(&msg_arc),  
                            info: Arc::clone(&info_arc),
                            state: Arc::clone(&state),
                        };
                        println!("Internal: {:?}", start.elapsed());
                        if let Err(e) = cmd.execute(ctx).await {
                            error!("Error executing {}: {}", cmd_name, e);
                        }                        
                    }
                }
            }
            
            let duration = start.elapsed();
            println!("Executed in {:?}", duration);
        }
        Event::GroupUpdate(update) => {
            match &update.action {
                GroupNotificationAction::Ephemeral{expiration, trigger: _} => {
                    println!("Group JID {}", &update.group_jid.user_base());
                    state.set_expiration(&update.group_jid.user_base(), *expiration);
                }
                _ => {}
            }
        }
        _ => {}
    }
}