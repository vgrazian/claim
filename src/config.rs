use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;
use directories::ProjectDirs;

#[derive(Serialize, Deserialize, Debug)]
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

    pub fn load() -> Result<Self, String> {
        let config_path = Self::get_config_path()
            .ok_or("Could not determine config directory")?;

        if !config_path.exists() {
            return Err("Config file does not exist".to_string());
        }

        let config_data = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        serde_json::from_str(&config_data)
            .map_err(|e| format!("Failed to parse config: {}", e))
    }

    pub fn save(&self) -> Result<(), String> {
        let config_path = Self::get_config_path()
            .ok_or("Could not determine config directory")?;

        // Create parent directories if they don't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let config_data = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&config_path, config_data)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(())
    }

    pub fn prompt_for_api_key() -> String {
        println!("Please enter your API key:");
        let mut api_key = String::new();
        io::stdin()
            .read_line(&mut api_key)
            .expect("Failed to read input");
        api_key.trim().to_string()
    }
}