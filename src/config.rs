use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;
use directories::ProjectDirs;
use anyhow::{Result, anyhow};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub api_key: String,
}

impl Config {
    pub fn new(api_key: String) -> Self {
        Config { api_key }
    }

    pub fn get_config_path() -> Option<PathBuf> {
        ProjectDirs::from("com", "yourname", "claim")
            .map(|proj_dirs| proj_dirs.config_dir().join("config.json"))
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()
            .ok_or_else(|| anyhow!("Could not determine config directory"))?;

        if !config_path.exists() {
            return Err(anyhow!("Config file does not exist"));
        }

        let config_data = fs::read_to_string(&config_path)
            .map_err(|e| anyhow!("Failed to read config file: {}", e))?;

        let config: Config = serde_json::from_str(&config_data)
            .map_err(|e| anyhow!("Failed to parse config: {}", e))?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()
            .ok_or_else(|| anyhow!("Could not determine config directory"))?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| anyhow!("Failed to create config directory: {}", e))?;
        }

        let config_data = serde_json::to_string_pretty(self)
            .map_err(|e| anyhow!("Failed to serialize config: {}", e))?;

        fs::write(&config_path, config_data)
            .map_err(|e| anyhow!("Failed to write config file: {}", e))?;

        Ok(())
    }

    pub fn prompt_for_api_key() -> String {
        println!("Please enter your Monday.com API key:");
        println!("You can find it at: https://your-account.monday.com/admin/integrations/api");
        println!("Note: Your API key will be stored securely in your system's config directory.");
        
        let mut api_key = String::new();
        io::stdin()
            .read_line(&mut api_key)
            .expect("Failed to read input");
        api_key.trim().to_string()
    }
}