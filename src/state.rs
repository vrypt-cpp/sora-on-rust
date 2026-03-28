use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

use crate::config::AppConfig;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ChatSettings {
    pub expiration: u32,
}

pub struct AppState {
    pub settings: DashMap<String, ChatSettings>,
    pub file_path: String,
    pub start_time: Instant,
    pub config: AppConfig,
    file_lock: Mutex<()>,
}

impl AppState {
    pub fn load(path: &str) -> Arc<Self> {
        let start_time = Instant::now();
        let file_path = path.to_string();        
        let settings = if let Ok(data) = fs::read_to_string(&file_path) {
            serde_json::from_str(&data).unwrap_or_else(|_| DashMap::new())
        } else {
            DashMap::new()
        };
        let config = AppConfig::load().unwrap();
        Arc::new(Self {
            settings,
            file_path,
            start_time,
            config,
            file_lock: Mutex::new(()),
        })
    }    

    pub async fn set_expiration(self: Arc<Self>, jid: String, expiration: u32) {
        if let Some(current) = self.settings.get(&jid) {
            if current.expiration == expiration {
                return;
            }
        }
        self.settings.insert(jid, ChatSettings { expiration });
        let state_clone = Arc::clone(&self);
        tokio::spawn(async move {
            let _lock = state_clone.file_lock.lock().await;
            
            let snapshot: HashMap<String, ChatSettings> = state_clone.settings
                .iter()
                .map(|r| (r.key().clone(), r.value().clone()))
                .collect();

            if let Ok(json) = serde_json::to_string_pretty(&snapshot) {
                if let Err(e) = tokio::fs::write(&state_clone.file_path, json).await {
                    log::error!("Unable to save state: {}", e);
                }
            }
        });
    }
    pub fn get_expiration(&self, jid: &str) -> u32 {
        self.settings.get(jid).map(|s| s.expiration).unwrap_or(0)
    }
}