mod config;
mod monday;

use config::Config;
use monday::{MondayClient, MondayUser, Item};
use anyhow::{Result, anyhow};
use std::process;
use chrono::prelude::*;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "claim")]
#[command(about = "Monday.com claim management tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Query claims from Monday.com board
    Query {
        /// Number of rows to display (default: 5)
        #[arg(short, long, default_value_t = 5)]
        limit: usize,
    },
    /// Add a new claim (placeholder for future implementation)
    Add {
        /// Claim description
        #[arg(short, long)]
        description: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match run(cli).await {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

async fn run(cli: Cli) -> Result<()> {
    // Load configuration
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

    let client = MondayClient::new(config.api_key.clone());
    let user = client.get_current_user().await?;
    let current_year = get_current_year().to_string();

    // Print user info with year
    println!("\nRunning for user id {}, user name {}, email {} for year {}",
        user.id, user.name, user.email, current_year);

    // Handle commands
    match cli.command {
        Some(Commands::Query { limit }) => {
            println!("Querying board for user's items (limit: {})...", limit);
            query_board(&client, &user, &current_year, limit).await?;
        }
        Some(Commands::Add { description }) => {
            println!("Add functionality is not yet implemented.");
            if let Some(desc) = description {
                println!("Would add claim: {}", desc);
            }
        }
        None => {
            // Default action when no command is provided
            println!("No command specified. Use --help for available commands.");
        }
    }

    Ok(())
}

async fn query_board(client: &MondayClient, user: &MondayUser, year: &str, limit: usize) -> Result<()> {
    let board_id = "6500270039";
    
    println!("Querying board {} for group '{}'...", board_id, year);
    
    let board = client.query_board(board_id, year, user.id, limit).await?;
    
    println!("\n=== Board: {} ===", board.name);
    
    // Display groups
    println!("\nAvailable groups:");
    if let Some(groups) = &board.groups {
        for group in groups {
            println!("  - {} (ID: {})", group.title, group.id);
        }
    } else {
        println!("  - No groups found in board");
    }
    
    // Display filtered items - look for any group that has items
    let mut found_items = false;
    if let Some(groups) = &board.groups {
        for group in groups {
            if let Some(ref items_page) = group.items_page {
                if !items_page.items.is_empty() {
                    found_items = true;
                    println!("\n=== FILTERED ITEMS for User {} ===", user.name);
                    println!("Found {} items for user {} in group '{}':", 
                            items_page.items.len(), user.name, group.title);
                    
                    for (index, item) in items_page.items.iter().enumerate() {
                        let item_name = item.name.as_deref().unwrap_or("Unnamed");
                        let item_id = item.id.as_deref().unwrap_or("Unknown");
                        println!("\n{}. {} (ID: {})", index + 1, item_name, item_id);
                        println!("   Columns:");
                        
                        for col in &item.column_values {
                            if let Some(value) = &col.value {
                                if value != "null" && !value.is_empty() {
                                    let col_id = col.id.as_deref().unwrap_or("Unknown");
                                    println!("     - {}: {}", col_id, value);
                                }
                            }
                        }
                    }
                    break; // Only show the first group with items
                }
            }
        }
    }
    
    if !found_items {
        println!("\nNo items found for user {} in any group.", user.name);
        println!("This means either:");
        println!("1. No items exist in any group");
        println!("2. Items exist but none are assigned to user ID {}", user.id);
        println!("3. The person column uses a different format than expected");
    }
    
    Ok(())
}

fn get_current_year() -> i32 {
    let now = Local::now();
    now.year()
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