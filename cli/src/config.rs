//! Configuration management for DagLock CLI.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

/// CLI configuration stored at ~/.daglock/config.toml
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub api_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_url: "http://localhost:8443".to_string(),
        }
    }
}

impl Config {
    pub fn path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".daglock").join("config.toml")
    }

    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            toml::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        info!("Config saved to {}", path.display());
        Ok(())
    }
}

pub async fn handle_config(api_url: Option<String>) -> anyhow::Result<()> {
    let mut config = Config::load();
    if let Some(url) = api_url {
        config.api_url = url;
        config.save()?;
    }
    println!("Current config:");
    println!("  API URL: {}", config.api_url);
    println!("  Config file: {}", Config::path().display());
    Ok(())
}
