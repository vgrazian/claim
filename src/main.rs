mod config;
mod monday;

use config::Config;
use monday::{MondayClient, MondayUser};
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
    /// Add a new claim
    Add {
        /// Date (YYYY-MM-DD format)
        #[arg(short = 'D', long)]
        date: Option<String>,
        
        /// Activity type (vacation, billable, holding, education, work_reduction, tbd, holiday, unknown, illness)
        #[arg(short = 't', long)]
        activity_type: Option<String>,
        
        /// Customer name
        #[arg(short = 'c', long)]
        customer: Option<String>,
        
        /// Work item
        #[arg(short = 'w', long)]
        work_item: Option<String>,
        
        /// Number of hours
        #[arg(short = 'H', long)]
        hours: Option<f64>,
        
        /// Number of working days (default: 1)
        #[arg(short = 'd', long)]
        days: Option<f64>,
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
        Some(Commands::Add { date, activity_type, customer, work_item, hours, days }) => {
            handle_add_command(&client, &user, &current_year, date, activity_type, customer, work_item, hours, days).await?;
        }
        None => {
            // Default action when no command is provided
            println!("No command specified. Use --help for available commands.");
        }
    }

    Ok(())
}

async fn handle_add_command(
    client: &MondayClient,
    user: &MondayUser,
    current_year: &str,
    date: Option<String>,
    activity_type: Option<String>,
    customer: Option<String>,
    work_item: Option<String>,
    hours: Option<f64>,
    days: Option<f64>,
) -> Result<()> {
    let (final_date, final_activity_type, final_customer, final_work_item, final_hours, final_days) = 
        if date.is_none() && activity_type.is_none() && customer.is_none() && 
           work_item.is_none() && hours.is_none() && days.is_none() {
        // Interactive mode - no parameters provided
        prompt_for_claim_details()?
    } else {
        // Command line mode - use provided parameters
        // Validate date if provided
        if let Some(ref d) = date {
            validate_date(d)?;
        }
        (date.unwrap_or_default(), activity_type, customer, work_item, hours, days)
    };
    
    // Validate that we have at least the date
    if final_date.is_empty() {
        return Err(anyhow!("Date is required for adding a claim"));
    }
    
    // Process activity type - default to "billable" if not provided
    let activity_type_str = final_activity_type.unwrap_or_else(|| "billable".to_string());
    let activity_type_value = map_activity_type_to_value(&activity_type_str);
    
    // Process days - default to 1.0 if not provided
    let days_value = final_days.unwrap_or(1.0);
    
    // Display the user info and the claim that would be added
    println!("\n=== Adding Claim for User ===");
    println!("User ID: {}, Name: {}, Email: {}", user.id, user.name, user.email);
    println!("Year: {}", current_year);
    println!("\n=== Claim Details ===");
    println!("Date: {}", final_date);
    println!("Activity Type: {} (value: {})", activity_type_str, activity_type_value);
    println!("Customer: {}", final_customer.as_deref().unwrap_or("Not specified"));
    println!("Work Item: {}", final_work_item.as_deref().unwrap_or("Not specified"));
    println!("Hours: {}", final_hours.map(|h| h.to_string()).unwrap_or_else(|| "Not specified".to_string()));
    println!("Days: {}", days_value);
    
    // Here you would typically call a function to actually add the item to Monday.com
    // For now, we'll just display the confirmation
    println!("\n✅ Claim would be added to Monday.com board");
    println!("Note: Actual Monday.com integration is not yet implemented");
    
    Ok(())
}

fn prompt_for_claim_details() -> Result<(String, Option<String>, Option<String>, Option<String>, Option<f64>, Option<f64>)> {
    use std::io::{self, Write};
    
    println!("\n=== Add New Claim ===");
    println!("Enter claim details (press Enter to skip optional fields):");
    
    // Date (mandatory)
    let mut date = String::new();
    loop {
        print!("Date (YYYY-MM-DD, YYYY.MM.DD, or YYYY/MM/DD, required): ");
        io::stdout().flush()?;
        date.clear();
        io::stdin().read_line(&mut date)?;
        date = date.trim().to_string();
        
        if date.is_empty() {
            println!("Date is required!");
            continue;
        }
        
        // Basic date validation with flexible separators
        if validate_date_flexible(&date).is_ok() {
            // Normalize the date to YYYY-MM-DD format
            date = normalize_date(&date);
            break;
        } else {
            println!("Invalid date format. Please use YYYY-MM-DD, YYYY.MM.DD, or YYYY/MM/DD format.");
        }
    }
    
    // Activity type (optional, defaults to billable)
    print!("Activity type (optional, default: billable): ");
    io::stdout().flush()?;
    let mut activity_type = String::new();
    io::stdin().read_line(&mut activity_type)?;
    let activity_type = activity_type.trim().to_string();
    let activity_type = if activity_type.is_empty() { None } else { Some(activity_type) };
    
    // Customer name (optional)
    print!("Customer name (optional): ");
    io::stdout().flush()?;
    let mut customer = String::new();
    io::stdin().read_line(&mut customer)?;
    let customer = customer.trim().to_string();
    let customer = if customer.is_empty() { None } else { Some(customer) };
    
    // Work item (optional)
    print!("Work item (optional): ");
    io::stdout().flush()?;
    let mut work_item = String::new();
    io::stdin().read_line(&mut work_item)?;
    let work_item = work_item.trim().to_string();
    let work_item = if work_item.is_empty() { None } else { Some(work_item) };
    
    // Hours (optional)
    print!("Number of hours (optional): ");
    io::stdout().flush()?;
    let mut hours = String::new();
    io::stdin().read_line(&mut hours)?;
    let hours = hours.trim();
    let hours = if hours.is_empty() {
        None
    } else {
        match hours.parse::<f64>() {
            Ok(h) => Some(h),
            Err(_) => {
                println!("Invalid number format for hours. Skipping.");
                None
            }
        }
    };
    
    // Days (optional, defaults to 1)
    print!("Number of working days (optional, default: 1): ");
    io::stdout().flush()?;
    let mut days = String::new();
    io::stdin().read_line(&mut days)?;
    let days = days.trim();
    let days = if days.is_empty() {
        None
    } else {
        match days.parse::<f64>() {
            Ok(d) => Some(d),
            Err(_) => {
                println!("Invalid number format for days. Skipping.");
                None
            }
        }
    };
    
    Ok((date, activity_type, customer, work_item, hours, days))
}

fn validate_date(date_str: &str) -> Result<()> {
    // Try multiple date formats
    let formats = ["%Y-%m-%d", "%Y.%m.%d", "%Y/%m/%d"];
    
    for format in &formats {
        if chrono::NaiveDate::parse_from_str(date_str, format).is_ok() {
            return Ok(());
        }
    }
    
    Err(anyhow!("Invalid date format: {}. Please use YYYY-MM-DD, YYYY.MM.DD, or YYYY/MM/DD format.", date_str))
}

fn validate_date_flexible(date_str: &str) -> Result<()> {
    validate_date(date_str)
}

fn normalize_date(date_str: &str) -> String {
    // Try to parse with different formats and return in YYYY-MM-DD format
    let formats = ["%Y-%m-%d", "%Y.%m.%d", "%Y/%m/%d"];
    
    for format in &formats {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, format) {
            return date.format("%Y-%m-%d").to_string();
        }
    }
    
    // If we can't parse it, return the original (this shouldn't happen if validate_date was called first)
    date_str.to_string()
}

fn map_activity_type_to_value(activity_type: &str) -> u8 {
    match activity_type.to_lowercase().as_str() {
        "vacation" => 0,
        "billable" => 1,
        "holding" => 2,
        "education" => 3,
        "work_reduction" => 4,
        "tbd" => 5,
        "holiday" => 6,
        "unknown" => 7,
        "illness" => 8,
        _ => {
            println!("Warning: Unknown activity type '{}', defaulting to billable (1)", activity_type);
            1 // Default to billable for unknown types
        }
    }
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