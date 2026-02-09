use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const CONFIG_FILE: &str = "config.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub path_tech_docs: String,
    pub path_production: String,
    pub path_offers: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            path_tech_docs: String::new(),
            path_production: String::new(),
            path_offers: String::new(),
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        if Path::new(CONFIG_FILE).exists() {
            if let Ok(content) = fs::read_to_string(CONFIG_FILE) {
                if let Ok(config) = serde_json::from_str(&content) {
                    return config;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = fs::write(CONFIG_FILE, content);
        }
    }
}