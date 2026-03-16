use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Clone)]
pub struct AppConfig {
    pub prefixes: Vec<String>,
    pub db_path: String,
    pub custom_code: String,

    #[serde(skip)]
    pub phone_number: String,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        let phone = std::env::var("PHONE_NUMBER").expect("PHONE_NUMBER must be set in .env");

        let toml_str = fs::read_to_string("Config.toml")?;
        let mut config: AppConfig = toml::from_str(&toml_str)?;
        
        config.phone_number = phone;
        Ok(config)
    }
}