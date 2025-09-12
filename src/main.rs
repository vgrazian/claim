mod config;
mod monday;

use config::Config;
use monday::{MondayClient, MondayUser};
use anyhow::{Result, anyhow};
use std::process;

#[tokio::main]
async fn main() {
    match run().await {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

// Update the error handling in src/main.rs
async fn run() -> Result<()> {
    // Try to load existing config
    let config = match Config::load() {
        Ok(config) => {
            println!("Loaded API key: {}", mask_api_key(&config.api_key));
            config
        }
        Err(_) => {
            println!("No API key found. Let's set one up!");
            let api_key = Config::prompt_for_api_key();
            
            if api_key.is_empty() {
                return Err(anyhow!("API key cannot be empty"));
            }

            let config = Config::new(api_key.clone());
            
            // Test the API key before saving
            println!("Testing connection to Monday.com...");
            let client = MondayClient::new(api_key);
            match client.test_connection().await {
                Ok(_) => {
                    config.save()?;
                    println!("API key validated and saved successfully!");
                    config
                }
                Err(e) => {
                    return Err(anyhow!("Failed to validate API key: {}. Please check your API key and try again.", e));
                }
            }
        }
    };

    // Connect to Monday.com and get user info
    println!("Connecting to Monday.com...");
    let client = MondayClient::new(config.api_key.clone());
    
    match client.get_current_user().await {
        Ok(user) => {
            display_user_info(&user);
            println!("Connection successful! Ready to process claims.");
        }
        Err(e) => {
            eprintln!("Detailed error: {}", e);
            return Err(anyhow!("Failed to connect to Monday.com. Please check:\n1. Your API key is correct\n2. You have internet connectivity\n3. Your Monday.com account has API access permissions"));
        }
    }

    Ok(())
}

fn display_user_info(user: &MondayUser) {
    println!("\n=== Monday.com User Information ===");
    println!("User ID: {}", user.id);
    println!("Name: {}", user.name);
    println!("Email: {}", user.email);
    println!("===================================");
}

fn mask_api_key(api_key: &str) -> String {
    if api_key.len() <= 8 {
        "*".repeat(api_key.len())
    } else {
        let visible_part = &api_key[..4];
        let masked_part = "*".repeat(api_key.len() - 4);
        format!("{}{}", visible_part, masked_part)
    }
}