use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;

use crate::config::AppConfig;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ChatSettings {
    pub expiration: u32,
}

pub struct AppState {
    pub settings: DashMap<String, ChatSettings>,
    pub db: sled::Db,
    pub start_time: Instant,
    pub config: Arc<AppConfig>,
}

impl AppState {
    pub fn load(config: Arc<AppConfig>) -> Arc<Self> {
        let start_time = Instant::now();
        let db = sled::open("database/chat").expect("Error opening sled database");
        let settings = DashMap::new();

        // hydration from db to cache
        for item in db.iter() {
            if let Ok((key, value)) = item {
                let jid = String::from_utf8_lossy(&key).to_string();

                if value.len() == 4 {
                    let bytes: [u8; 4] = value.as_ref().try_into().unwrap();
                    let expiration = u32::from_be_bytes(bytes);
                    settings.insert(jid, ChatSettings { expiration });
                }
            }
        }

        Arc::new(Self {
            settings,
            db,
            start_time,
            config: config,
        })
    }    

    pub async fn set_expiration(self: Arc<Self>, jid: String, expiration: u32) {
        if let Some(current) = self.settings.get(&jid) {
            if current.expiration == expiration {
                return;
            }
        }
        let jid_db = jid.clone();
        self.settings.insert(jid, ChatSettings { expiration });
        let state_clone = Arc::clone(&self);
        tokio::spawn(async move {
            let val_bytes = expiration.to_be_bytes();
            if let Err(e) = state_clone.db.insert(jid_db, &val_bytes) {
                log::error!("Error inserting data into sled database: {}", e);
            }
        });
    }
    pub fn get_expiration(&self, jid: &str) -> u32 {
        self.settings.get(jid).map(|s| s.expiration).unwrap_or(0)
    }
}