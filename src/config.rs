use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub api_key: String,
}

impl Config {
    pub fn new(api_key: String) -> Self {
        Config { api_key }
    }

    pub fn get_config_path() -> Option<std::path::PathBuf> {
        ProjectDirs::from("com", "vgrazian", "claim")
            .map(|proj_dirs| proj_dirs.config_dir().join("config.json"))
    }

    /// Load API key from secure keyring storage
    /// Falls back to file-based storage for backward compatibility
    pub fn load() -> Result<Self> {
        // Try to load from keyring first
        match Self::load_from_keyring() {
            Ok(config) => Ok(config),
            Err(_) => {
                // Fall back to file-based storage for backward compatibility
                match Self::load_from_file() {
                    Ok(config) => {
                        // Migrate to keyring
                        if let Err(e) = config.save_to_keyring() {
                            eprintln!(
                                "Warning: Failed to migrate API key to secure storage: {}",
                                e
                            );
                        } else {
                            println!("✅ API key migrated to secure system keyring");
                            // Optionally remove old file
                            if let Some(path) = Self::get_config_path() {
                                let _ = std::fs::remove_file(path);
                            }
                        }
                        Ok(config)
                    }
                    Err(e) => Err(e),
                }
            }
        }
    }

    /// Load API key from system keyring
    fn load_from_keyring() -> Result<Self> {
        let entry = Entry::new("claim", "monday_api_key")
            .map_err(|e| anyhow!("Failed to access keyring: {}", e))?;

        let api_key = entry
            .get_password()
            .map_err(|e| anyhow!("Failed to retrieve API key from keyring: {}", e))?;

        Ok(Config { api_key })
    }

    /// Load API key from file (legacy method for backward compatibility)
    fn load_from_file() -> Result<Self> {
        let config_path = Self::get_config_path()
            .ok_or_else(|| anyhow!("Could not determine config directory"))?;

        if !config_path.exists() {
            return Err(anyhow!("Config file does not exist"));
        }

        let config_data = std::fs::read_to_string(&config_path)
            .map_err(|e| anyhow!("Failed to read config file: {}", e))?;

        let config: Config = serde_json::from_str(&config_data)
            .map_err(|e| anyhow!("Failed to parse config: {}", e))?;

        Ok(config)
    }

    /// Save API key to secure keyring storage
    pub fn save(&self) -> Result<()> {
        self.save_to_keyring()
    }

    /// Save API key to system keyring
    fn save_to_keyring(&self) -> Result<()> {
        let entry = Entry::new("claim", "monday_api_key")
            .map_err(|e| anyhow!("Failed to access keyring: {}", e))?;

        entry
            .set_password(&self.api_key)
            .map_err(|e| anyhow!("Failed to save API key to keyring: {}", e))?;

        Ok(())
    }

    /// Save API key to file (legacy method, kept for testing)
    #[allow(dead_code)]
    fn save_to_file(&self) -> Result<()> {
        let config_path = Self::get_config_path()
            .ok_or_else(|| anyhow!("Could not determine config directory"))?;

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| anyhow!("Failed to create config directory: {}", e))?;
        }

        let config_data = serde_json::to_string_pretty(self)
            .map_err(|e| anyhow!("Failed to serialize config: {}", e))?;

        std::fs::write(&config_path, config_data)
            .map_err(|e| anyhow!("Failed to write config file: {}", e))?;

        Ok(())
    }

    pub fn prompt_for_api_key() -> String {
        println!("Please enter your Monday.com API key:");
        println!("You can find it at: https://your-account.monday.com/admin/integrations/api");
        println!("Note: Your API key will be stored securely in your system's keyring.");

        let mut api_key = String::new();
        io::stdin()
            .read_line(&mut api_key)
            .expect("Failed to read input");
        api_key.trim().to_string()
    }

    // Test-specific methods that allow specifying the path
    #[cfg(test)]
    pub fn load_from_path(path: &std::path::Path) -> Result<Self> {
        if !path.exists() {
            return Err(anyhow!("Config file does not exist"));
        }

        let config_data = std::fs::read_to_string(path)
            .map_err(|e| anyhow!("Failed to read config file: {}", e))?;

        let config: Config = serde_json::from_str(&config_data)
            .map_err(|e| anyhow!("Failed to parse config: {}", e))?;

        Ok(config)
    }

    #[cfg(test)]
    pub fn save_to_path(&self, path: &std::path::Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| anyhow!("Failed to create config directory: {}", e))?;
        }

        let config_data = serde_json::to_string_pretty(self)
            .map_err(|e| anyhow!("Failed to serialize config: {}", e))?;

        std::fs::write(path, config_data)
            .map_err(|e| anyhow!("Failed to write config file: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    fn setup_test_env() -> TempDir {
        let temp_dir = TempDir::new().unwrap();

        // Clear any existing directory environment variables that might interfere
        env::remove_var("XDG_CONFIG_HOME");
        env::remove_var("XDG_DATA_HOME");
        env::remove_var("XDG_CACHE_HOME");
        env::remove_var("HOME");
        env::remove_var("APPDATA");
        env::remove_var("LOCALAPPDATA");

        // Set up proper environment for directories crate based on OS
        if cfg!(target_os = "windows") {
            env::set_var("APPDATA", temp_dir.path());
        } else if cfg!(target_os = "macos") {
            env::set_var("HOME", temp_dir.path());
        } else {
            // Linux and other Unix-like systems
            env::set_var("HOME", temp_dir.path());
        }

        temp_dir
    }

    #[test]
    fn test_config_new() {
        let config = Config::new("test-api-key".to_string());
        assert_eq!(config.api_key, "test-api-key");
    }

    #[test]
    fn test_config_save_and_load_with_direct_path() {
        let temp_dir = setup_test_env();
        let test_config_path = temp_dir.path().join("test-config.json");

        let config = Config::new("test-api-key".to_string());

        // Test saving to specific path
        let save_result = config.save_to_path(&test_config_path);
        assert!(save_result.is_ok(), "Save failed: {:?}", save_result.err());

        // Verify file was created
        assert!(test_config_path.exists(), "Config file should exist");

        // Test loading from the same path
        let loaded_config = Config::load_from_path(&test_config_path);
        assert!(
            loaded_config.is_ok(),
            "Load failed: {:?}",
            loaded_config.err()
        );

        let loaded = loaded_config.unwrap();
        assert_eq!(loaded.api_key, "test-api-key");
    }

    #[test]
    fn test_config_load_nonexistent() {
        let temp_dir = setup_test_env();
        let nonexistent_path = temp_dir.path().join("nonexistent-config.json");

        let result = Config::load_from_path(&nonexistent_path);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("does not exist"));
    }

    #[test]
    fn test_config_save_creates_directory() {
        let temp_dir = setup_test_env();
        let nested_config_path = temp_dir
            .path()
            .join("nested")
            .join("dir")
            .join("config.json");

        let config = Config::new("test-api-key".to_string());
        assert!(config.save_to_path(&nested_config_path).is_ok());

        // Verify the config file was created
        assert!(nested_config_path.exists(), "Config file should exist");

        // Verify the parent directories were created
        let parent_dir = nested_config_path.parent().unwrap();
        assert!(parent_dir.exists(), "Parent directory should exist");
    }

    #[test]
    fn test_get_config_path() {
        let _temp_dir = setup_test_env();

        let path = Config::get_config_path();
        assert!(path.is_some());

        let path_str = path.unwrap().to_string_lossy().to_string();
        if cfg!(target_os = "windows") {
            assert!(
                path_str.contains("AppData"),
                "Path should contain AppData: {}",
                path_str
            );
        } else if cfg!(target_os = "macos") {
            assert!(
                path_str.contains("Library"),
                "Path should contain Library: {}",
                path_str
            );
        } else {
            assert!(
                path_str.contains(".config") || path_str.contains("claim"),
                "Path should contain .config or claim: {}",
                path_str
            );
        }
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::new("test-api-key".to_string());

        // Test serialization
        let serialized = serde_json::to_string(&config);
        assert!(serialized.is_ok());
        let json_str = serialized.unwrap();
        assert!(json_str.contains("test-api-key"));

        // Test deserialization
        let deserialized: Result<Config, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());
        assert_eq!(deserialized.unwrap().api_key, "test-api-key");
    }

    #[test]
    fn test_config_round_trip() {
        let temp_dir = setup_test_env();
        let config_path = temp_dir.path().join("round-trip-config.json");

        // Create and save config
        let original_config = Config::new("round-trip-test-key".to_string());
        assert!(original_config.save_to_path(&config_path).is_ok());

        // Load it back
        let loaded_config = Config::load_from_path(&config_path).expect("Should load config");
        assert_eq!(loaded_config.api_key, "round-trip-test-key");

        // Modify and save again
        let modified_config = Config::new("modified-key".to_string());
        assert!(modified_config.save_to_path(&config_path).is_ok());

        // Load again to verify change
        let reloaded_config =
            Config::load_from_path(&config_path).expect("Should load modified config");
        assert_eq!(reloaded_config.api_key, "modified-key");
    }

    #[test]
    fn test_config_with_empty_api_key() {
        let config = Config::new("".to_string());
        assert_eq!(config.api_key, "");
    }

    #[test]
    fn test_config_with_special_characters() {
        let special_key = "key-with-special-chars!@#$%^&*()";
        let config = Config::new(special_key.to_string());
        assert_eq!(config.api_key, special_key);

        // Test serialization/deserialization with special characters
        let temp_dir = setup_test_env();
        let config_path = temp_dir.path().join("special-config.json");

        assert!(config.save_to_path(&config_path).is_ok());
        let loaded_config = Config::load_from_path(&config_path).unwrap();
        assert_eq!(loaded_config.api_key, special_key);
    }
}

// Made with Bob
