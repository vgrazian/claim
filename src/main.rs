mod config;

use config::Config;
use std::process;

fn main() {
    match run() {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn run() -> Result<(), String> {
    // Try to load existing config
    match Config::load() {
        Ok(config) => {
            println!("Loaded API key: {}", mask_api_key(&config.api_key));
            println!("Using API key for claims processing...");
            // Here you would use the API key for your actual functionality
            process_claims(&config.api_key)?;
        }
        Err(_) => {
            println!("No API key found. Let's set one up!");
            let api_key = Config::prompt_for_api_key();
            
            if api_key.is_empty() {
                return Err("API key cannot be empty".to_string());
            }

            let config = Config::new(api_key);
            config.save()?;
            
            println!("API key saved successfully!");
            println!("Using API key for claims processing...");
            process_claims(&config.api_key)?;
        }
    }

    Ok(())
}

fn process_claims(api_key: &str) -> Result<(), String> {
    // Your actual claim processing logic would go here
    println!("Processing claims with API key: {}", mask_api_key(api_key));
    // Simulate some work
    println!("Claims processed successfully!");
    Ok(())
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