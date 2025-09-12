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
    
    // Display groups - now handling Option<Vec<Group>>
    println!("\nAvailable groups:");
    if let Some(groups) = &board.groups {
        for group in groups {
            println!("  - {} (ID: {})", group.title, group.id);
        }
    } else {
        println!("  - No groups found in board");
    }
    
    // Display items if available - now using board.items instead of board.items_page
    if let Some(ref items) = board.items {
        println!("\n=== Items in Group '{}' (Limit: {}) ===", year, limit);
        
        if items.is_empty() {
            println!("No items found in this group.");
        } else {
            let user_items: Vec<&Item> = items.iter()
                .filter(|item| is_user_item(item, user.id))
                .take(limit)
                .collect();
            
            if user_items.is_empty() {
                println!("No items found for user {} in this group.", user.name);
            } else {
                println!("Found {} items for user {}:", user_items.len(), user.name);
                for (index, item) in user_items.iter().enumerate() {
                    println!("\n{}. {} (ID: {})", index + 1, item.name, item.id);
                    println!("   Columns:");
                    
                    for col in &item.column_values {
                        if !col.text.is_empty() && col.text != "null" {
                            println!("     - {} ({}): {}", col.column_type, col.id, col.text);
                        }
                    }
                }
            }
        }
    } else {
        println!("\nNo items found in the board.");
    }
    
    // Display available columns from first item (if any)
    if let Some(ref items) = board.items {
        if let Some(first_item) = items.first() {
            println!("\n=== Available Columns ===");
            for col in &first_item.column_values {
                println!("  - {} ({}): {}", col.column_type, col.id, col.text);
            }
        }
    }
    
    Ok(())
}

// Helper function to filter items by user
// This is a placeholder - you'll need to adjust based on how users are stored in your items
fn is_user_item(item: &Item, user_id: i64) -> bool {
    // This is a simple implementation - you might need to check specific columns
    // that contain user information. For now, we'll return all items since we
    // don't know which column contains the user assignment.
    
    // Look for user information in column values
    for col in &item.column_values {
        // Check if this column might contain user information
        if col.text.contains(&user_id.to_string()) {
            return true;
        }
        
        // You might need to adjust this based on your board's column structure
        // Common patterns: user IDs, email addresses, or person column values
        if col.column_type.to_lowercase().contains("person") 
            || col.text.to_lowercase().contains("user")
            || col.text.to_lowercase().contains("assign") {
            if col.text.contains(&user_id.to_string()) 
                || col.text.contains("valerio") 
                || col.text.contains("graziani") 
                || col.text.contains("valerio.graziani") {
                return true;
            }
        }
    }
    
    // If we can't determine ownership, show all items for debugging
    true
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