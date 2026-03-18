use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ChatSettings {
    pub expiration: u32,
}

pub struct AppState {
    pub settings: DashMap<String, ChatSettings>,
    pub file_path: String,
}

impl AppState {
    pub fn load(path: &str) -> Arc<Self> {
        let file_path = path.to_string();        
        let settings = if let Ok(data) = fs::read_to_string(&file_path) {
            serde_json::from_str(&data).unwrap_or_else(|_| DashMap::new())
        } else {
            DashMap::new()
        };

        Arc::new(Self {
            settings,
            file_path,
        })
    }

    pub fn set_expiration(&self, jid: &str, exp: u32) {
    self.settings.entry(jid.to_string())
            .and_modify(|s| s.expiration = exp)
            .or_insert(ChatSettings { expiration: exp });
        let file_path = self.file_path.clone();        
        let settings_snapshot: std::collections::HashMap<String, ChatSettings> = 
            self.settings.iter().map(|entry| (entry.key().clone(), entry.value().clone())).collect();
        tokio::spawn(async move {
            if let Ok(data) = serde_json::to_string_pretty(&settings_snapshot) {
                if let Err(e) = std::fs::write(file_path, data) {
                    eprintln!("Unable to save JSON: {}", e);
                }
            }
        });        
    }

    pub fn get_expiration(&self, jid: &str) -> u32 {
        self.settings.get(jid).map(|s| s.expiration).unwrap_or(0)
    }
}