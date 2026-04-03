use log::{error, info};
use wacore::stanza::GroupNotificationAction;
use wacore::types::events::GroupUpdate;
use wacore::types::message::MessageInfo;
use std::sync::Arc;
use crate::state::AppState;
use wacore::{client::context::SendContextResolver, types::events::Event};
use whatsapp_rust::client::Client;
use crate::config::AppConfig;
use crate::utils::MessageExt;
use tokio::sync::RwLock;
use std::sync::LazyLock;
use chrono::Utc;

static SUPERUSER_LID: LazyLock<RwLock<Option<String>>> = LazyLock::new(|| RwLock::new(None));

pub async fn event_handler(event: Event, client: Arc<Client>, config: Arc<AppConfig>, state: Arc<AppState>) {
    match event {
        Event::Connected(_) => handle_connected(config, client).await,
        Event::Message(msg, info) => handle_message(*msg, client, config, info, state).await,
        Event::GroupUpdate(update) => handle_group_exp(update, state).await,
        _ => {}
    }
}


async fn handle_connected(config: Arc<AppConfig>, client: Arc<Client>) {
    let current_name = client.get_push_name().await;
    if current_name.is_empty() {
        let _ = client.profile().set_push_name("sora-on-rust").await;
    }

    let _ = client.presence().set_available().await;
    info!("✅ Bot connected successfully!");
    if let Some(su_pn) = &config.superuser {
        let mut found_lid = client.get_lid_for_phone(su_pn).await.map(|j| j.to_string());
        println!("{:?}",&found_lid);
        if found_lid.is_none() {
            match client.contacts().get_info(&[su_pn.as_str()]).await {
                Ok(contacts) => {
                    if let Some(contact) = contacts.into_iter().next() {
                        if let Some(lid) = contact.lid {
                            found_lid = Some(lid.user);
                            println!("{:?}",&found_lid);
                        }
                    }
                }
                Err(e) => log::error!("Unable retrieve contact info from server: {}", e),
            }
        }
        println!("{:?}",&found_lid);
        if let Some(lid) = found_lid {
            
            let mut lock = SUPERUSER_LID.write().await;
            *lock = Some(lid);
        } else {
            log::warn!("Unable to get LID for superuser: {}", su_pn);
        }
    }
}

async fn handle_message(msg: waproto::whatsapp::Message, client: Arc<Client>, config: Arc<AppConfig>, info: MessageInfo, state: Arc<AppState> ) {
            // println!("{:#?}", msg);
            // let start = std::time::Instant::now();
            if let Some(exp) = msg.get_expiration_timer() {
                state.clone().set_expiration(info.source.chat.to_string(), exp);
                // println!("Expiration received: {}", exp);
            }
            
            if let Some(text) = msg.text_content() {
                let matched_prefix: Option<&String> = config.prefixes.iter().find(|p| text.starts_with(*p));
                let prefix = match matched_prefix {
                    Some(p) => p.to_string(),
                    None => return,
                };
                let cmd_name = text.strip_prefix(&prefix).unwrap_or(text).split_whitespace().next().unwrap_or("").to_lowercase();
                // println!("{}", &info_arc.source.sender.user);
                let msg_timestamp = Utc::now() - &info.timestamp;
                if &msg_timestamp.to_std().unwrap_or_default() > &state.start_time.elapsed() {return;}
                if let Some(cmd) = crate::commands::cmd::COMMAND_MAP.get(&cmd_name) {
                    let privileged = is_privileged(info.source.sender.user.as_str(), &info, &config).await;
                    let category = cmd.category();
                    if config.mode == "self" {
                        if !privileged {
                            println!("{}", &info.source.sender.user);
                            println!("Not privileged");
                            return;
                        }
                    }
                    if category == "root" && !privileged {
                        println!("Permission denied");
                        return
                    };
                    tokio::spawn(async move {
                        let _ = client.chatstate().send_composing(&info.source.chat).await;
                        let base = msg.text_content().map(|t| t.strip_prefix(&prefix).unwrap_or(t)).unwrap_or("");
                        let args: Vec<&str> = base.split_whitespace().skip(1).collect();
                        let body = base.strip_prefix(base.split_whitespace().next().unwrap_or("")).unwrap_or("").trim();
                        
                        let ctx = crate::commands::cmd::Context {
                            client: Arc::clone(&client),
                            msg: &msg,
                            info: &info,
                            state: Arc::clone(&state),
                            args: &args,
                            body: body,
                        };
                        let _ = client.chatstate().send_paused(&info.source.chat).await;
                        if let Err(e) = cmd.execute(ctx).await {
                            error!("Error executing command: {}", e);
                        }
                    });
                }
            }
            
            //let duration = start.elapsed();
            //println!("Executed in {:?}", duration);
}

async fn handle_group_exp(update: GroupUpdate, state: Arc<AppState>) {
    match &update.action {
        GroupNotificationAction::Ephemeral{expiration, trigger: _} => {
            state.set_expiration(update.group_jid.to_string(), *expiration);
        }
        _ => {}
    }
}


async fn is_privileged(sender: &str, info: &MessageInfo, config: &Arc<AppConfig>) -> bool {
    let me = info.source.is_from_me;
    let su = if info.source.sender.is_lid() {
        let lock = SUPERUSER_LID.read().await;
        lock.as_deref() == Some(sender)
    } else {
        config.superuser.as_deref() == Some(sender)
    };

    me || su
}
