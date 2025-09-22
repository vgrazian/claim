mod config;
mod monday;

use config::Config;
use monday::{MondayClient, MondayUser};
use anyhow::{Result, anyhow};
use std::process;
use chrono::prelude::*;
use clap::{Parser, Subcommand};
use serde_json::json;
use std::io;

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
        
        /// Date to filter claims (YYYY-MM-DD, YYYY.MM.DD, or YYYY/MM/DD format)
        #[arg(short = 'D', long)]
        date: Option<String>,
        
        /// Verbose output
        #[arg(short = 'v', long)]
        verbose: bool,
    },
    /// Add a new claim
    Add {
        /// Date (YYYY-MM-DD format)
        #[arg(short = 'D', long)]
        date: Option<String>,
        
        /// Activity type (vacation, billable, holding, education, work_reduction, tbd, holiday, presales, illness, boh1, boh2, boh3)
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
        
        /// Number of working days (default: 1, skips weekends)
        #[arg(short = 'd', long)]
        days: Option<f64>,
        
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
        
        /// Verbose output
        #[arg(short = 'v', long)]
        verbose: bool,
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
    // Determine if verbose mode is enabled
    let verbose = match &cli.command {
        Some(Commands::Query { verbose, .. }) => *verbose,
        Some(Commands::Add { verbose, .. }) => *verbose,
        None => false,
    };

    // Load configuration
    let config = match Config::load() {
        Ok(config) => {
            if verbose {
                println!("Loaded API key: {}", mask_api_key(&config.api_key));
            }
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
            match client.test_connection_verbose(verbose).await {
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
    let user = client.get_current_user_verbose(verbose).await?;
    let current_year = get_current_year().to_string();

    // Print user info with year
    println!("\nRunning for user id {}, user name {}, email {} for year {}",
        user.id, user.name, user.email, current_year);

    // Handle commands
    match cli.command {
        Some(Commands::Query { limit, date, verbose }) => {
            if verbose {
                println!("Querying board for user's items (limit: {})...", limit);
            }
            query_board(&client, &user, &current_year, limit, date, verbose).await?;
        }
        Some(Commands::Add { date, activity_type, customer, work_item, hours, days, yes, verbose }) => {
            handle_add_command(&client, &user, &current_year, date, activity_type, customer, work_item, hours, days, yes, verbose).await?;
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
    yes: bool,
    verbose: bool,
) -> Result<()> {
    let (final_date, final_activity_type, final_customer, final_work_item, final_hours, final_days, is_interactive) = 
        if date.is_none() && activity_type.is_none() && customer.is_none() && 
           work_item.is_none() && hours.is_none() && days.is_none() {
        // Interactive mode - no parameters provided
        let (d, at, c, wi, h, d_val) = prompt_for_claim_details()?;
        (d, at, c, wi, h, d_val, true)
    } else {
        // Command line mode - use provided parameters
        // Validate date if provided
        if let Some(ref d) = date {
            validate_date(d)?;
        }
        (date.unwrap_or_default(), activity_type, customer, work_item, hours, days, false)
    };
    
    // If date is not provided, default to today's date
    let final_date = if final_date.is_empty() {
        Local::now().format("%Y-%m-%d").to_string()
    } else {
        final_date
    };
    
    // Process activity type - default to "billable" if not provided
    let activity_type_str = final_activity_type.unwrap_or_else(|| "billable".to_string());
    let activity_type_value = map_activity_type_to_value(&activity_type_str);
    
    // Process days - default to 1.0 if not provided
    let days_value = final_days.unwrap_or(1.0);
    
    // Calculate the actual dates (skipping weekends)
    let start_date = chrono::NaiveDate::parse_from_str(&final_date, "%Y-%m-%d")?;
    let target_days = days_value as i64;
    let actual_dates = calculate_working_dates(start_date, target_days);
    
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
    println!("Days requested: {}", days_value);
    println!("Actual working days: {}", actual_dates.len());
    
    // Show which dates will be used
    println!("\n📅 Dates that will be created (weekends skipped):");
    for (i, date) in actual_dates.iter().enumerate() {
        let weekday = date.format("%A");
        println!("  {}. {} ({})", i + 1, date.format("%Y-%m-%d"), weekday);
    }
    
    // Get the current year's group ID from the board
    let board = client.query_board_verbose("6500270039", current_year, user.id, 1, verbose).await?;
    let group_id = get_year_group_id(&board, current_year);
    
    if verbose {
        println!("\n🔍 Verbose mode: Found group '{}' with ID: {}", current_year, group_id);
        show_graphql_mutations(&actual_dates, &activity_type_value, &final_customer, &final_work_item, final_hours, user.id, &user.name, &group_id);
    } else {
        println!("\nFound group '{}' with ID: {}", current_year, group_id);
    }
    
    // Ask for confirmation before proceeding (unless -y flag is used)
    if !yes {
        println!("\n🚀 Ready to create {} item(s) on Monday.com", actual_dates.len());
        println!("Do you want to proceed? (y/N)");
        
        let mut confirmation = String::new();
        io::stdin().read_line(&mut confirmation)?;
        
        if confirmation.trim().to_lowercase() != "y" {
            println!("Operation cancelled.");
            return Ok(());
        }
    } else {
        println!("\n🚀 Creating {} item(s) on Monday.com (skipping confirmation)", actual_dates.len());
    }
    
    // Actually create the items on Monday.com
    create_items_on_monday(
        client,
        &actual_dates,
        activity_type_value,
        &final_customer,
        &final_work_item,
        final_hours,
        user.id,
        &user.name,
        &group_id,
        verbose
    ).await?;
    
    // If this was interactive mode, show the equivalent command line
    if is_interactive {
        show_equivalent_command(&final_date, &activity_type_str, &final_customer, &final_work_item, final_hours, days_value, yes, verbose);
    }
    
    Ok(())
}

async fn create_items_on_monday(
    client: &MondayClient,
    actual_dates: &[NaiveDate],
    activity_type_value: u8,
    customer: &Option<String>,
    work_item: &Option<String>,
    hours: Option<f64>,
    user_id: i64,
    user_name: &str,
    group_id: &str,
    verbose: bool,
) -> Result<()> {
    let board_id = "6500270039";
    let mut successful_creations = 0;
    
    println!("\n🔄 Creating items on Monday.com...");
    
    for (i, date) in actual_dates.iter().enumerate() {
        let date_str = date.format("%Y-%m-%d").to_string();
        
        let mut column_values = json!({});
        
        // Set person column
        column_values["person"] = json!({
            "personsAndTeams": [
                {
                    "id": user_id,
                    "kind": "person"
                }
            ]
        });
        
        // Set date column
        column_values["date4"] = json!({
            "date": date_str
        });
        
        // Set activity type column
        column_values["status"] = json!({
            "index": activity_type_value
        });
        
        // Set customer name if provided
        if let Some(c) = customer {
            if !c.is_empty() {
                column_values["text__1"] = json!(c);
            }
        }
        
        // Set work item if provided
        if let Some(wi) = work_item {
            if !wi.is_empty() {
                column_values["text8__1"] = json!(wi);
            }
        }
        
        // Set hours if provided
        if let Some(h) = hours {
            column_values["numbers__1"] = json!(h.to_string());
        }
        
        if verbose {
            println!("\n📋 GraphQL Mutation for {} ({} of {}):", date_str, i + 1, actual_dates.len());
            let mutation = format!(
                r#"mutation {{
    create_item(
        board_id: "{}",
        group_id: "{}",
        item_name: "{}",
        column_values: "{}"
    ) {{
        id
    }}
}}"#,
                board_id,
                group_id,
                user_name,
                column_values.to_string().replace('"', "\\\"")
            );
            println!("{}", mutation);
        }
        
        println!("Creating item for {} ({} of {})...", date_str, i + 1, actual_dates.len());
        
        match client.create_item_verbose(board_id, group_id, user_name, &column_values, verbose).await {
            Ok(item_id) => {
                if verbose {
                    println!("✅ Successfully created item with response: {}", item_id);
                } else {
                    println!("✅ Successfully created item");
                }
                successful_creations += 1;
            }
            Err(e) => {
                println!("❌ Failed to create item for {}: {}", date_str, e);
                // Continue with other items even if one fails
            }
        }
        
        // Add a small delay to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }
    
    println!("\n🎉 Successfully created {} out of {} items", successful_creations, actual_dates.len());
    
    if successful_creations < actual_dates.len() {
        return Err(anyhow!("Some items failed to create. Check the errors above."));
    }
    
    Ok(())
}

fn show_graphql_mutations(actual_dates: &[NaiveDate], activity_type_value: &u8, customer: &Option<String>, work_item: &Option<String>, hours: Option<f64>, user_id: i64, user_name: &str, group_id: &str) {
    println!("\n📋 GraphQL Mutations that would be executed:");
    
    let board_id = "6500270039";
    
    for (i, date) in actual_dates.iter().enumerate() {
        let date_str = date.format("%Y-%m-%d").to_string();
        
        let mut column_values = json!({});
        
        // Set person column
        column_values["person"] = json!({
            "personsAndTeams": [
                {
                    "id": user_id,
                    "kind": "person"
                }
            ]
        });
        
        // Set date column
        column_values["date4"] = json!({
            "date": date_str
        });
        
        // Set activity type column
        column_values["status"] = json!({
            "index": activity_type_value
        });
        
        // Set customer name if provided
        if let Some(c) = customer {
            if !c.is_empty() {
                column_values["text__1"] = json!(c);
            }
        }
        
        // Set work item if provided
        if let Some(wi) = work_item {
            if !wi.is_empty() {
                column_values["text8__1"] = json!(wi);
            }
        }
        
        // Set hours if provided
        if let Some(h) = hours {
            column_values["numbers__1"] = json!(h.to_string());
        }
        
        let mutation = format!(
            r#"// Mutation for {} ({} of {})
mutation {{
    create_item(
        board_id: "{}",
        group_id: "{}",
        item_name: "{}",
        column_values: "{}"
    ) {{
        id
    }}
}}
"#,
            date_str,
            i + 1,
            actual_dates.len(),
            board_id,
            group_id,
            user_name,
            column_values.to_string().replace('"', "\\\"")
        );
        
        println!("{}", mutation);
    }
}

fn get_year_group_id(board: &monday::Board, year: &str) -> String {
    if let Some(groups) = &board.groups {
        for group in groups {
            if group.title == year {
                return group.id.clone();
            }
        }
    }
    // Fallback to a default group ID if not found
    "new_group_mkkbbd2q".to_string()
}

async fn query_board(
    client: &MondayClient,
    user: &MondayUser,
    year: &str,
    limit: usize,
    date: Option<String>,
    verbose: bool,
) -> Result<()> {
    let board_id = "6500270039";
    
    // Handle date filtering if provided
    let normalized_date = if let Some(ref date_str) = date {
        // Validate the date format
        validate_date(date_str)?;
        Some(normalize_date(date_str))
    } else {
        None
    };
    
    if verbose {
        if let Some(ref d) = normalized_date {
            println!("Querying board {} for group '{}' with date filter: {}...", board_id, year, d);
        } else {
            println!("Querying board {} for group '{}'...", board_id, year);
        }
    }
    
    let board = client.query_board_verbose(board_id, year, user.id, limit, verbose).await?;
    
    if verbose {
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
    }
    
    // Display filtered items - look for any group that has items
    let mut found_items = false;
    if let Some(groups) = &board.groups {
        for group in groups {
            if let Some(ref items_page) = group.items_page {
                if !items_page.items.is_empty() {
                    found_items = true;
                    println!("\n=== FILTERED ITEMS for User {} ===", user.name);
                    
                    // Show date filter info if applicable
                    if let Some(ref filter_date) = normalized_date {
                        println!("Date filter: {}", filter_date);
                    }
                    
                    println!("Found {} items for user {} in group '{}':", 
                            items_page.items.len(), user.name, group.title);
                    
                    for (index, item) in items_page.items.iter().enumerate() {
                        let item_name = item.name.as_deref().unwrap_or("Unnamed");
                        let item_id = item.id.as_deref().unwrap_or("Unknown");
                        println!("\n{}. {} (ID: {})", index + 1, item_name, item_id);
                        
                        // Display column values with titles in a formatted way
                        if !item.column_values.is_empty() {
                            println!("   Columns:");
                            // Find the maximum column title length for formatting
                            let max_title_len = item.column_values.iter()
                                .map(|col| {
                                    let col_id = col.id.as_deref().unwrap_or("");
                                    map_column_title(col_id).len()
                                })
                                .max()
                                .unwrap_or(0);
                            
                            for col in &item.column_values {
                                if let Some(col_id) = &col.id {
                                    let column_title = map_column_title(col_id);
                                    
                                    if let Some(value) = &col.value {
                                        if value != "null" && !value.is_empty() {
                                            // Format with aligned columns
                                            println!("     {:<width$} : {}", column_title, value, width = max_title_len);
                                        }
                                    } else if let Some(text) = &col.text {
                                        if !text.is_empty() && text != "null" {
                                            // Format with aligned columns
                                            println!("     {:<width$} : {}", column_title, text, width = max_title_len);
                                        }
                                    }
                                }
                            }
                        } else {
                            println!("   No column values available");
                        }
                    }
                    break; // Only show the first group with items
                }
            }
        }
    }
    
    if !found_items {
        println!("\nNo items found for user {} in any group.", user.name);
        if let Some(ref filter_date) = normalized_date {
            println!("Date filter: {}", filter_date);
        }
        println!("This means either:");
        println!("1. No items exist in any group");
        println!("2. Items exist but none are assigned to user ID {}", user.id);
        if let Some(_) = normalized_date {
            println!("3. No items match the date filter");
        } else {
            println!("3. The person column uses a different format than expected");
        }
    }
    
    Ok(())
}

fn map_column_title(column_id: &str) -> &str {
    match column_id {
        "subitems__1" => "Subitems",
        "person" => "Person",
        "status" => "Status",
        "date4" => "Date",
        "text__1" => "Text",
        "text8__1" => "Text 8", 
        "numbers__1" => "Numbers",
        "hours" => "Hours",
        "days" => "Days",
        "activity_type" => "Activity Type",
        "customer" => "Customer",
        "work_item" => "Work Item",
        _ => column_id, // Fall back to the ID if no mapping found
    }
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
        "presales" => 7,
        "illness" => 8,
        "boh1" => 9,
        "boh2" => 10,
        "boh3" => 11,
        _ => {
            println!("Warning: Unknown activity type '{}', defaulting to billable (1)", activity_type);
            1 // Default to billable for unknown types
        }
    }
}

fn calculate_working_dates(start_date: NaiveDate, target_days: i64) -> Vec<NaiveDate> {
    let mut dates = Vec::new();
    let mut current_date = start_date;
    let mut days_added = 0;
    
    while days_added < target_days {
        // Check if it's a weekday (Monday = 1, Friday = 5)
        let weekday = current_date.weekday().number_from_monday();
        if weekday <= 5 {
            dates.push(current_date);
            days_added += 1;
        }
        
        // Move to next day
        current_date = current_date + chrono::Duration::days(1);
    }
    
    dates
}

fn show_equivalent_command(date: &str, activity_type: &str, customer: &Option<String>, work_item: &Option<String>, hours: Option<f64>, days: f64, yes: bool, verbose: bool) {
    println!("\n💡 Equivalent command line:");
    
    let mut command_parts = Vec::new();
    command_parts.push(format!("claim add -D {}", date));
    
    // Only include activity type if it's not the default "billable"
    if activity_type != "billable" {
        command_parts.push(format!("-t {}", activity_type));
    }
    
    if let Some(c) = customer {
        if !c.is_empty() {
            command_parts.push(format!("-c \"{}\"", c));
        }
    }
    
    if let Some(wi) = work_item {
        if !wi.is_empty() {
            command_parts.push(format!("-w \"{}\"", wi));
        }
    }
    
    if let Some(h) = hours {
        command_parts.push(format!("-H {}", h));
    }
    
    // Only include days if it's not the default 1.0
    if (days - 1.0).abs() > f64::EPSILON {
        command_parts.push(format!("-d {}", days));
    }
    
    // Include -y flag if it would be needed
    if yes {
        command_parts.push("-y".to_string());
    }
    
    // Include -v flag if verbose
    if verbose {
        command_parts.push("-v".to_string());
    }
    
    println!("   {}", command_parts.join(" "));
}

fn prompt_for_claim_details() -> Result<(String, Option<String>, Option<String>, Option<String>, Option<f64>, Option<f64>)> {
    use std::io::{self, Write};
    
    println!("\n=== Add New Claim ===");
    println!("Enter claim details (press Enter to skip optional fields):");
    
    // Date (optional, defaults to today)
    let mut date = String::new();
    print!("Date (YYYY-MM-DD, YYYY.MM.DD, or YYYY/MM/DD, optional - default: today): ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut date)?;
    date = date.trim().to_string();
    
    // If date is provided, validate it
    if !date.is_empty() {
        // Basic date validation with flexible separators
        if validate_date_flexible(&date).is_ok() {
            // Normalize the date to YYYY-MM-DD format
            date = normalize_date(&date);
        } else {
            println!("Invalid date format. Please use YYYY-MM-DD, YYYY.MM.DD, or YYYY/MM/DD format.");
            return Err(anyhow!("Invalid date format"));
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
    print!("Number of working days (optional, default: 1, skips weekends): ");
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