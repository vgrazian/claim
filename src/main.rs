mod config;
mod monday;

use config::Config;
use monday::{MondayClient, MondayUser, Item};
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
        
        /// Date to filter claims (YYYY-MM-DD, YYYY.MM.DD, or YYYY/MM/DD format, default: today)
        #[arg(short = 'D', long)]
        date: Option<String>,
        
        /// Number of days to query (default: 1, skips weekends)
        #[arg(short = 'd', long, default_value_t = 1)]
        days: usize,
        
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
        Some(Commands::Query { limit, date, days, verbose }) => {
            if verbose {
                if date.is_none() {
                    println!("Querying board for user's items (limit: {}, days: {}, date: today)...", limit, days);
                } else {
                    println!("Querying board for user's items (limit: {}, days: {})...", limit, days);
                }
            }
            query_board(&client, &user, limit, date, days, verbose).await?;
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
    limit: usize,
    date: Option<String>,
    days: usize,
    verbose: bool,
) -> Result<()> {
    let board_id = "6500270039";
    
    // Handle date filtering - default to today if no date provided
    let (start_date, target_days) = if let Some(ref date_str) = date {
        // Validate the date format
        validate_date(date_str)?;
        let normalized_date = normalize_date(date_str);
        let start_date = chrono::NaiveDate::parse_from_str(&normalized_date, "%Y-%m-%d")?;
        (Some(start_date), days)
    } else {
        // Default to today's date
        let today = Local::now().naive_local().date();
        (Some(today), days)
    };
    
    // Calculate the date range if start date is provided
    let date_range = if let Some(start_date) = start_date {
        calculate_working_dates(start_date, target_days as i64)
    } else {
        Vec::new()
    };
    
    if verbose {
        if let Some(start_date_val) = start_date {
            if target_days > 1 {
                let end_date = date_range.last().map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default();
                println!("Querying board {} for user '{}' with date range: {} to {} ({} working days)...", 
                    board_id, user.name, 
                    start_date_val.format("%Y-%m-%d"),
                    end_date,
                    target_days);
            } else {
                println!("Querying board {} for user '{}' with date filter: {}...", 
                    board_id, user.name, start_date_val.format("%Y-%m-%d"));
            }
        } else {
            println!("Querying board {} for user '{}'...", board_id, user.name);
        }
    } else {
        // Show brief info even in non-verbose mode
        if let Some(start_date_val) = start_date {
            if target_days > 1 {
                let end_date = date_range.last().map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default();
                println!("Querying date range: {} to {} ({} days)...", 
                    start_date_val.format("%Y-%m-%d"),
                    end_date,
                    target_days);
            } else {
                println!("Querying date: {}...", start_date_val.format("%Y-%m-%d"));
            }
        }
    }
    
    // Use a special method to query by user name instead of user ID
    // Increased limit to catch all possible items
    let items = client.query_items_by_user_name(board_id, "new_group_mkkbbd2q", &user.name, 1000, verbose).await?;
    
    if verbose {
        println!("\n=== Raw items found for user: {} ===", items.len());
        // Debug: show all dates found
        for (i, item) in items.iter().enumerate() {
            if let Some(item_date) = extract_item_date(item) {
                println!("Item {}: {}", i + 1, item_date);
            } else {
                println!("Item {}: [No date found]", i + 1);
            }
        }
    }
    
    // Filter items by date range if date filter is provided
    let filtered_items: Vec<&Item> = if start_date.is_some() {
        if !date_range.is_empty() {
            let filtered: Vec<&Item> = items.iter()
                .filter(|item| is_item_matching_date_range(item, &date_range))
                .collect();
            
            if verbose {
                println!("After date range filtering: {} items", filtered.len());
                for item in &filtered {
                    if let Some(item_date) = extract_item_date(item) {
                        println!("Filtered item date: {}", item_date);
                    }
                }
            }
            filtered
        } else {
            items.iter().collect()
        }
    } else {
        items.iter().collect()
    };
    
    let limited_items: Vec<&Item> = filtered_items.iter().take(limit).cloned().collect();
    let limited_items_len = limited_items.len();
    let filtered_items_len = filtered_items.len();
    
    if verbose {
        println!("\n=== Filtered items matching date criteria: {} ===", filtered_items_len);
    }
    
    // Display the results in a simplified table format for multi-day queries
    if !filtered_items.is_empty() {
        if let Some(start_date_val) = start_date {
            if target_days > 1 {
                // Multi-day query - show simplified table
                // Use filtered_items to see all matching items
                display_simplified_table(&filtered_items, &date_range, &user.name, verbose);
            } else {
                // Single day query - show detailed format
                display_detailed_items(&limited_items, start_date, &user.name, filtered_items_len, limit);
            }
        } else {
            // No date filter - show detailed format
            display_detailed_items(&limited_items, None, &user.name, filtered_items_len, limit);
        }
    } else {
        println!("\nNo items found for user '{}'", user.name);
        if let Some(start_date_val) = start_date {
            if target_days > 1 {
                let end_date = date_range.last().map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default();
                println!("Date range: {} to {} ({} working days)", 
                    start_date_val.format("%Y-%m-%d"),
                    end_date,
                    target_days);
            } else {
                println!("Date filter: {}", start_date_val.format("%Y-%m-%d"));
            }
        }
        println!("This means either:");
        println!("1. No items exist for this user for the specified date(s)");
        println!("2. The user name in Monday.com differs from '{}'", user.name);
    }
    
    if let Some(start_date_val) = start_date {
        if target_days > 1 {
            let end_date = date_range.last().map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default();
            println!("\n✅ Found {} total items matching date range: {} to {}", 
                filtered_items_len, 
                start_date_val.format("%Y-%m-%d"),
                end_date);
        } else {
            println!("\n✅ Found {} total items matching date filter: {}", 
                filtered_items_len, start_date_val.format("%Y-%m-%d"));
        }
    }
    
    Ok(())
}
// Helper function to map activity type value back to name
fn map_activity_value_to_name(value: u8) -> String {
    match value {
        0 => "vacation".to_string(),
        1 => "billable".to_string(),
        2 => "holding".to_string(),
        3 => "education".to_string(),
        4 => "work_reduction".to_string(),
        5 => "tbd".to_string(),
        6 => "holiday".to_string(),
        7 => "presales".to_string(),
        8 => "illness".to_string(),
        9 => "boh1".to_string(),
        10 => "boh2".to_string(),
        11 => "boh3".to_string(),
        _ => format!("unknown({})", value),
    }
}

// Helper function to check if an item matches the specified date
fn is_item_matching_date(item: &Item, target_date: &str) -> bool {
    for col in &item.column_values {
        if let Some(col_id) = &col.id {
            if col_id == "date4" {
                // Parse the date column value to check if it matches the target date
                if let Some(value) = &col.value {
                    if let Ok(parsed_value) = serde_json::from_str::<serde_json::Value>(value) {
                        if let Some(date_obj) = parsed_value.get("date") {
                            if let Some(date_str) = date_obj.as_str() {
                                // Compare the date part only (ignore time if present)
                                if date_str.starts_with(target_date) {
                                    return true;
                                }
                            }
                        }
                    }
                }
                // Also check the text field as fallback
                if let Some(text) = &col.text {
                    if text.starts_with(target_date) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

// Helper function to check if an item matches any date in the range
fn is_item_matching_date_range(item: &Item, date_range: &[NaiveDate]) -> bool {
    for date in date_range {
        let date_str = date.format("%Y-%m-%d").to_string();
        if is_item_matching_date(item, &date_str) {
            return true;
        }
    }
    false
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
    if api_key.len() <= 4 {
        "*".repeat(api_key.len())
    } else {
        let visible_part = &api_key[..4];
        let masked_part = "*".repeat(api_key.len() - 4);
        format!("{}{}", visible_part, masked_part)
    }
}

// Helper function to extract date from an item
fn extract_item_date(item: &Item) -> Option<String> {
    for col in &item.column_values {
        if let Some(col_id) = &col.id {
            if col_id == "date4" {
                // Try to parse from value field (JSON format)
                if let Some(value) = &col.value {
                    if value != "null" && !value.is_empty() {
                        if let Ok(parsed_value) = serde_json::from_str::<serde_json::Value>(value) {
                            if let Some(date_obj) = parsed_value.get("date") {
                                if let Some(date_str) = date_obj.as_str() {
                                    // Normalize the date format to YYYY-MM-DD
                                    if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                                        return Some(naive_date.format("%Y-%m-%d").to_string());
                                    }
                                    // Try other common date formats
                                    if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(date_str, "%Y/%m/%d") {
                                        return Some(naive_date.format("%Y-%m-%d").to_string());
                                    }
                                    if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(date_str, "%Y.%m.%d") {
                                        return Some(naive_date.format("%Y-%m-%d").to_string());
                                    }
                                    // Return the original string if parsing fails
                                    return Some(date_str.to_string());
                                }
                            }
                        }
                    }
                }
                // Fallback: try to parse from text field
                if let Some(text) = &col.text {
                    if !text.is_empty() && text != "null" {
                        // Normalize the date format
                        if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(text, "%Y-%m-%d") {
                            return Some(naive_date.format("%Y-%m-%d").to_string());
                        }
                        if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(text, "%Y/%m/%d") {
                            return Some(naive_date.format("%Y-%m-%d").to_string());
                        }
                        if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(text, "%Y.%m.%d") {
                            return Some(naive_date.format("%Y-%m-%d").to_string());
                        }
                        return Some(text.to_string());
                    }
                }
            }
        }
    }
    None
}

// Helper function to extract specific column value
fn extract_column_value(item: &Item, column_id: &str) -> String {
    for col in &item.column_values {
        if let Some(col_id) = &col.id {
            if col_id == column_id {
                if let Some(value) = &col.value {
                    if value != "null" && !value.is_empty() {
                        // Try to parse JSON value for complex columns
                        if let Ok(parsed_value) = serde_json::from_str::<serde_json::Value>(value) {
                            if let Some(text) = parsed_value.as_str() {
                                return text.to_string();
                            }
                        }
                        return value.to_string();
                    }
                }
                if let Some(text) = &col.text {
                    if !text.is_empty() && text != "null" {
                        return text.to_string();
                    }
                }
            }
        }
    }
    "".to_string()
}

// Helper function to extract status value and map it to activity type name
fn extract_status_value(item: &Item) -> String {
    for col in &item.column_values {
        if let Some(col_id) = &col.id {
            if col_id == "status" {
                // Try to parse the status value from JSON
                if let Some(value) = &col.value {
                    if let Ok(parsed_value) = serde_json::from_str::<serde_json::Value>(value) {
                        if let Some(status_index) = parsed_value.get("index") {
                            if let Some(index_num) = status_index.as_u64() {
                                return map_activity_value_to_name(index_num as u8);
                            }
                        }
                    }
                }
                // Fallback: try to parse directly from text
                if let Some(text) = &col.text {
                    if !text.is_empty() && text != "null" {
                        return text.to_string();
                    }
                }
            }
        }
    }
    "unknown".to_string()
}

// Helper function to truncate strings for table display
fn truncate_string(s: &str, max_length: usize) -> String {
    if s.len() <= max_length {
        s.to_string()
    } else {
        format!("{}...", &s[..max_length.saturating_sub(3)])
    }
}

// Display simplified table for multi-day queries
fn display_simplified_table(items: &[&Item], date_range: &[NaiveDate], user_name: &str, verbose: bool) {
    println!("\n=== CLAIMS SUMMARY for User {} ===", user_name);
    
    let start_date = date_range.first().map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default();
    let end_date = date_range.last().map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default();
    
    println!("Date Range: {} to {}", start_date, end_date);
    
    if verbose {
        // Debug: Show what dates we found in the items
        println!("Debug: Processing {} items with dates:", items.len());
        for (i, item) in items.iter().enumerate() {
            if let Some(item_date) = extract_item_date(item) {
                println!("  Item {}: {}", i + 1, item_date);
            } else {
                println!("  Item {}: [No date found]", i + 1);
            }
        }
        
        // Add detailed debug for date extraction
        println!("=== DEBUG: Verifying date extraction for all {} items ===", items.len());
        let mut success_count = 0;
        for (i, item) in items.iter().enumerate() {
            if let Some(date) = extract_item_date(item) {
                success_count += 1;
                println!("  Item {}: ✓ Extracted date: {}", i + 1, date);
            } else {
                println!("  Item {}: ✗ Failed to extract date", i + 1);
                // Debug the item structure
                println!("    Item ID: {:?}", item.id);
                println!("    Item name: {:?}", item.name);
                println!("    Column count: {}", item.column_values.len());
                for (j, col) in item.column_values.iter().enumerate() {
                    println!("    Column {}: ID={:?}, Value={:?}, Text={:?}", 
                        j + 1, col.id, col.value, col.text);
                }
            }
        }
        println!("Date extraction success: {}/{}", success_count, items.len());
    }
    
    // Create a table header with Status column
    println!("\n{:<12} {:<12} {:<20} {:<15} {:<6}", "Date", "Status", "Customer", "Work Item", "Hours");
    println!("{}", "-".repeat(70));
    
    // Group items by date using a HashMap
    let mut items_by_date: std::collections::HashMap<String, Vec<&Item>> = std::collections::HashMap::new();
    
    for item in items {
        if let Some(item_date) = extract_item_date(item) {
            // Ensure the date is in YYYY-MM-DD format
            let normalized_date = match chrono::NaiveDate::parse_from_str(&item_date, "%Y-%m-%d") {
                Ok(date) => date.format("%Y-%m-%d").to_string(),
                Err(_) => item_date, // Use as-is if parsing fails
            };
            items_by_date.entry(normalized_date).or_insert_with(Vec::new).push(item);
        }
    }
    
    if verbose {
        // Debug: Show what dates we have in the map
        let mut dates: Vec<String> = items_by_date.keys().cloned().collect();
        dates.sort();
        println!("Debug: Unique dates in HashMap: {:?}", dates);
        println!("Debug: Date range to display: {:?}", date_range.iter().map(|d| d.format("%Y-%m-%d").to_string()).collect::<Vec<_>>());
    }
    
    // Display items in date range order
    let mut total_hours: f64 = 0.0;
    let mut displayed_items = 0;
    
    for date in date_range {
        let date_str = date.format("%Y-%m-%d").to_string();
        
        if let Some(date_items) = items_by_date.get(&date_str) {
            displayed_items += date_items.len();
            for item in date_items {
                let status = extract_status_value(item);
                let customer = extract_column_value(item, "text__1");
                let work_item = extract_column_value(item, "text8__1");
                let hours_str = extract_column_value(item, "numbers__1");
                let hours = hours_str.parse::<f64>().unwrap_or(0.0);
                total_hours += hours;
                
                println!("{:<12} {:<12} {:<20} {:<15} {:<6}", 
                    date_str, 
                    truncate_string(&status, 10),
                    truncate_string(&customer, 18),
                    truncate_string(&work_item, 13),
                    hours_str);
            }
        } else {
            // Show empty row for dates with no entries
            println!("{:<12} {:<12} {:<20} {:<15} {:<6}", date_str, "-", "-", "-", "-");
        }
    }
    
    println!("{}", "-".repeat(70));
    println!("{:<12} {:<12} {:<20} {:<15} {:<6.1}", 
        "TOTAL", "", "", "", total_hours);
    println!("\nFound {} items across {} days", displayed_items, date_range.len());
}

// Helper function to display detailed items (original format)
fn display_detailed_items(items: &[&Item], filter_date: Option<NaiveDate>, user_name: &str, filtered_items_len: usize, limit: usize) {
    println!("\n=== FILTERED ITEMS for User {} ===", user_name);
    
    if let Some(date) = filter_date {
        println!("Date filter: {}", date.format("%Y-%m-%d"));
    }
    
    println!("Found {} items for user {}:", filtered_items_len, user_name);
    
    for (index, item) in items.iter().enumerate() {
        let item_name = item.name.as_deref().unwrap_or("Unnamed");
        let item_id = item.id.as_deref().unwrap_or("Unknown");
        println!("\n{}. {} (ID: {})", index + 1, item_name, item_id);
        
        if !item.column_values.is_empty() {
            println!("   Columns:");
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
                            println!("     {:<width$} : {}", column_title, value, width = max_title_len);
                        }
                    } else if let Some(text) = &col.text {
                        if !text.is_empty() && text != "null" {
                            println!("     {:<width$} : {}", column_title, text, width = max_title_len);
                        }
                    }
                }
            }
        } else {
            println!("   No column values available");
        }
    }
    
    if filtered_items_len > limit {
        println!("\n... and {} more items (showing first {} items)", 
               filtered_items_len - limit, limit);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::env;

    fn setup_test_env() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        
        // Set up environment for directories crate
        if cfg!(target_os = "windows") {
            env::set_var("APPDATA", temp_dir.path());
        } else {
            env::set_var("HOME", temp_dir.path());
        }
        
        temp_dir
    }

    #[test]
    fn test_mask_api_key() {
        // Test the masking behavior for different length API keys
        
        // For API keys longer than 4 characters: first 4 visible, rest masked
        let result = mask_api_key("12345678");
        assert_eq!(result.len(), 8); // Same length as input
        assert_eq!(&result[0..4], "1234"); // First 4 characters visible
        assert!(result.chars().skip(4).all(|c| c == '*')); // Rest are asterisks
        
        // For API keys exactly 4 characters: all masked
        let result = mask_api_key("1234");
        assert_eq!(result, "****"); // All characters masked
        
        // For API keys shorter than 4 characters: all masked
        let result = mask_api_key("123");
        assert_eq!(result, "***"); // All characters masked
        assert_eq!(mask_api_key("12"), "**");
        assert_eq!(mask_api_key("1"), "*");
        
        // Edge case: empty string
        assert_eq!(mask_api_key(""), "");
        
        // Test with a realistic API key
        let api_key = "abcdefghijklmnop";
        let masked = mask_api_key(api_key);
        assert_eq!(masked.len(), api_key.len());
        assert_eq!(&masked[0..4], "abcd"); // First 4 visible
        assert!(masked.chars().skip(4).all(|c| c == '*')); // Rest masked
        
        // Verify the original and masked are different (security check)
        assert_ne!(api_key, masked);
    }

    #[test]
    fn test_normalize_date() {
        assert_eq!(normalize_date("2025-09-15"), "2025-09-15");
        assert_eq!(normalize_date("2025.09.15"), "2025-09-15");
        assert_eq!(normalize_date("2025/09/15"), "2025-09-15");
        // Test that invalid dates return the original string
        assert_eq!(normalize_date("invalid"), "invalid");
    }

    #[test]
    fn test_validate_date() {
        assert!(validate_date("2025-09-15").is_ok());
        assert!(validate_date("2025.09.15").is_ok());
        assert!(validate_date("2025/09/15").is_ok());
        assert!(validate_date("invalid-date").is_err());
    }

    #[test]
    fn test_map_activity_type_to_value() {
        assert_eq!(map_activity_type_to_value("billable"), 1);
        assert_eq!(map_activity_type_to_value("vacation"), 0);
        assert_eq!(map_activity_type_to_value("holding"), 2);
        assert_eq!(map_activity_type_to_value("unknown"), 1); // defaults to billable
    }

    #[test]
    fn test_map_activity_value_to_name() {
        assert_eq!(map_activity_value_to_name(1), "billable");
        assert_eq!(map_activity_value_to_name(0), "vacation");
        assert_eq!(map_activity_value_to_name(2), "holding");
        assert_eq!(map_activity_value_to_name(99), "unknown(99)");
    }

    #[test]
    fn test_calculate_working_dates() {
        let start_date = NaiveDate::from_ymd_opt(2025, 9, 15).unwrap(); // Monday
        let dates = calculate_working_dates(start_date, 5);
        
        assert_eq!(dates.len(), 5);
        // Should skip weekends - Monday to Friday
        assert_eq!(dates[0].weekday(), Weekday::Mon);
        assert_eq!(dates[1].weekday(), Weekday::Tue);
        assert_eq!(dates[2].weekday(), Weekday::Wed);
        assert_eq!(dates[3].weekday(), Weekday::Thu);
        assert_eq!(dates[4].weekday(), Weekday::Fri);
    }

    #[test]
    fn test_calculate_working_dates_with_weekend() {
        let start_date = NaiveDate::from_ymd_opt(2025, 9, 13).unwrap(); // Saturday
        let dates = calculate_working_dates(start_date, 3);
        
        assert_eq!(dates.len(), 3);
        // Should skip Saturday and Sunday, start on Monday
        assert_eq!(dates[0].weekday(), Weekday::Mon);
        assert_eq!(dates[1].weekday(), Weekday::Tue);
        assert_eq!(dates[2].weekday(), Weekday::Wed);
    }

    #[test]
    fn test_map_column_title() {
        assert_eq!(map_column_title("date4"), "Date");
        assert_eq!(map_column_title("person"), "Person");
        assert_eq!(map_column_title("status"), "Status");
        assert_eq!(map_column_title("text__1"), "Text");
        assert_eq!(map_column_title("unknown"), "unknown");
    }

    #[test]
    fn test_get_current_year() {
        let year = get_current_year();
        let current_year = Local::now().year();
        assert_eq!(year, current_year);
    }

    #[test]
    fn test_is_item_matching_date() {
        let mut item = Item::default();
        let mut date_column = monday::ColumnValue::default();
        date_column.id = Some("date4".to_string());
        date_column.value = Some(r#"{"date": "2025-09-15"}"#.to_string());
        item.column_values.push(date_column);

        assert!(is_item_matching_date(&item, "2025-09-15"));
        assert!(!is_item_matching_date(&item, "2025-09-16"));
    }

    #[test]
    fn test_is_item_matching_date_with_text() {
        let mut item = Item::default();
        let mut date_column = monday::ColumnValue::default();
        date_column.id = Some("date4".to_string());
        date_column.text = Some("2025-09-15".to_string());
        item.column_values.push(date_column);

        assert!(is_item_matching_date(&item, "2025-09-15"));
        assert!(!is_item_matching_date(&item, "2025-09-16"));
    }

    #[test]
    fn test_cli_parsing_query() {
        let result = Cli::try_parse_from(&["claim", "query", "-D", "2025-09-15"]);
        assert!(result.is_ok());
        
        let cli = result.unwrap();
        match cli.command {
            Some(Commands::Query { date, .. }) => {
                assert_eq!(date, Some("2025-09-15".to_string()));
            }
            _ => panic!("Expected Query command"),
        }
    }

    #[test]
    fn test_cli_parsing_add() {
        let result = Cli::try_parse_from(&["claim", "add", "-c", "test"]);
        assert!(result.is_ok());
        
        let cli = result.unwrap();
        match cli.command {
            Some(Commands::Add { customer, .. }) => {
                assert_eq!(customer, Some("test".to_string()));
            }
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_cli_parsing_query_with_days() {
        let result = Cli::try_parse_from(&["claim", "query", "-D", "2025-09-15", "-d", "5"]);
        assert!(result.is_ok());
        
        let cli = result.unwrap();
        match cli.command {
            Some(Commands::Query { date, days, .. }) => {
                assert_eq!(date, Some("2025-09-15".to_string()));
                assert_eq!(days, 5);
            }
            _ => panic!("Expected Query command"),
        }
    }

    #[test]
    fn test_show_equivalent_command() {
        // Capture the output or just test that it doesn't panic
        show_equivalent_command(
            "2025-09-15",
            "billable",
            &Some("Customer".to_string()),
            &Some("WorkItem".to_string()),
            Some(8.0),
            1.0,
            false,
            false
        );
        // If we get here without panic, the test passes
        assert!(true);
    }

    #[test]
    fn test_validate_date_flexible() {
        assert!(validate_date_flexible("2025-09-15").is_ok());
        assert!(validate_date_flexible("2025.09.15").is_ok());
        assert!(validate_date_flexible("2025/09/15").is_ok());
        assert!(validate_date_flexible("invalid-date").is_err());
    }

    #[test]
    fn test_extract_item_date() {
        let mut item = Item::default();
        let mut date_column = monday::ColumnValue::default();
        date_column.id = Some("date4".to_string());
        date_column.value = Some(r#"{"date": "2025-09-15"}"#.to_string());
        item.column_values.push(date_column);

        let extracted_date = extract_item_date(&item);
        assert_eq!(extracted_date, Some("2025-09-15".to_string()));
    }

    #[test]
    fn test_extract_column_value() {
        let mut item = Item::default();
        let mut text_column = monday::ColumnValue::default();
        text_column.id = Some("text__1".to_string());
        text_column.value = Some("Test Customer".to_string());
        item.column_values.push(text_column);

        let extracted_value = extract_column_value(&item, "text__1");
        assert_eq!(extracted_value, "Test Customer");
    }

    #[test]
    fn test_extract_status_value() {
        let mut item = Item::default();
        let mut status_column = monday::ColumnValue::default();
        status_column.id = Some("status".to_string());
        status_column.value = Some(r#"{"index": 1}"#.to_string());
        item.column_values.push(status_column);

        let extracted_status = extract_status_value(&item);
        assert_eq!(extracted_status, "billable");
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("short", 10), "short");
        assert_eq!(truncate_string("very long string", 10), "very lo...");
        assert_eq!(truncate_string("exact", 5), "exact");
        assert_eq!(truncate_string("tiny", 2), "...");
    }
}

// Helper function for testing
#[cfg(test)]
fn create_test_item_with_date(date: &str) -> Item {
    let mut item = Item::default();
    let mut date_column = monday::ColumnValue::default();
    date_column.id = Some("date4".to_string());
    date_column.value = Some(format!(r#"{{"date": "{}"}}"#, date));
    item.column_values.push(date_column);
    item
}