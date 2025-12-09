mod add;
mod config;
mod delete;
mod monday;
mod query;
mod selenium;
mod time;
mod utils;

use anyhow::{anyhow, Result};
use chrono::Datelike;
use clap::{Parser, Subcommand};
use config::Config;
use monday::MondayClient;
use std::process;

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
        #[arg(short = 'D', long = "date")]
        date: Option<String>,

        /// Number of days to query (default: 1, skips weekends)
        #[arg(short = 'd', long = "days", default_value_t = 1)]
        days: usize,

        /// Customer name to filter by
        #[arg(short = 'c', long = "customer")] // NEW: Customer filter for query
        customer: Option<String>,

        /// Work item to filter by
        #[arg(short = 'w', long = "wi")] // NEW: Work item filter for query
        work_item: Option<String>,

        /// Verbose output
        #[arg(short = 'v', long = "verbose")]
        verbose: bool,
    },
    /// Add a new claim
    Add {
        /// Date (YYYY-MM-DD format)
        #[arg(short = 'D', long = "date")]
        date: Option<String>,

        /// Activity type (vacation, billable, holding, education, work_reduction, tbd, holiday, presales, illness, paid_not_worked, intellectual_capital, business_development, overhead)
        #[arg(short = 't', long = "type")]
        activity_type: Option<String>,

        /// Customer name
        #[arg(short = 'c', long = "customer")]
        customer: Option<String>,

        /// Work item
        #[arg(short = 'w', long = "wi")]
        work_item: Option<String>,

        /// Number of hours
        #[arg(short = 'H', long = "hours")]
        hours: Option<f64>,

        /// Number of working days (default: 1, skips weekends)
        #[arg(short = 'd', long = "days")]
        days: Option<f64>,

        /// Comment for the claim
        #[arg(short = 'k', long = "comment")] // NEW: Comment parameter
        comment: Option<String>,

        /// Skip confirmation prompt
        #[arg(short = 'y', long = "yes")]
        yes: bool,

        /// Verbose output
        #[arg(short = 'v', long = "verbose")]
        verbose: bool,
    },
    /// Delete a claim item by ID
    Delete {
        /// Item ID to delete
        #[arg(short = 'x', long = "id")]
        delete_id: String,

        /// Skip confirmation prompt
        #[arg(short = 'y', long = "yes")]
        yes: bool,

        /// Verbose output
        #[arg(short = 'v', long = "verbose")]
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
        Some(Commands::Delete { verbose, .. }) => *verbose,
        None => false,
    };

    // Load configuration
    let config = match Config::load() {
        Ok(config) => {
            if verbose {
                println!("Loaded API key: {}", utils::mask_api_key(&config.api_key));
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
                    return Err(anyhow!(
                        "Failed to validate API key: {}. Please check your API key and try again.",
                        e
                    ));
                }
            }
        }
    };

    let client = MondayClient::new(config.api_key.clone());
    let user = client.get_current_user_verbose(verbose).await?;
    let current_year = utils::get_current_year().to_string();

    // Print user info with year
    println!(
        "\nRunning for user id {}, user name {}, email {} for year {}",
        user.id, user.name, user.email, current_year
    );

    // Handle commands
    match cli.command {
        Some(Commands::Query {
            limit,
            date,
            days,
            customer,  // NEW: Pass customer filter
            work_item, // NEW: Pass work item filter
            verbose,
        }) => {
            query::handle_query_command(
                &client, &user, limit, date, days, customer, work_item, verbose,
            )
            .await?;
        }
        Some(Commands::Add {
            date,
            activity_type,
            customer,
            work_item,
            hours,
            days,
            comment, // NEW: Pass comment
            yes,
            verbose,
        }) => {
            add::handle_add_command(
                &client,
                &user,
                &current_year,
                date,
                activity_type,
                customer,
                work_item,
                hours,
                days,
                comment, // NEW: Pass comment
                yes,
                verbose,
            )
            .await?;
        }
        Some(Commands::Delete {
            delete_id,
            yes,
            verbose,
        }) => {
            delete::handle_delete_command(&client, &user, delete_id, yes, verbose).await?;
        }
        None => {
            // Default action when no command is provided
            println!("No command specified. Use --help for available commands.");
        }
    }

    Ok(())
}

// Re-export utility functions for use in other modules
pub use utils::{
    calculate_working_dates, map_activity_type_to_value, map_activity_value_to_name, mask_api_key,
    normalize_date, truncate_string, validate_date,
};

// Helper function to get year group ID (used by add module)
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

// Keep only the essential helper functions used by the add module
fn show_equivalent_command(
    date: &str,
    activity_type: &str,
    customer: &Option<String>,
    work_item: &Option<String>,
    comment: &Option<String>, // NEW: Comment parameter
    hours: Option<f64>,
    days: f64,
    yes: bool,
    verbose: bool,
) {
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

    // NEW: Include comment if provided
    if let Some(cmt) = comment {
        if !cmt.is_empty() {
            command_parts.push(format!("--comment \"{}\"", cmt));
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

fn prompt_for_claim_details() -> Result<(
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<f64>,
    Option<f64>,
    Option<String>, // NEW: Return comment
)> {
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
            println!(
                "Invalid date format. Please use YYYY-MM-DD, YYYY.MM.DD, or YYYY/MM/DD format."
            );
            return Err(anyhow!("Invalid date format"));
        }
    }

    // Activity type (optional, defaults to billable)
    print!("Activity type (optional, default: billable): ");
    io::stdout().flush()?;
    let mut activity_type = String::new();
    io::stdin().read_line(&mut activity_type)?;
    let activity_type = activity_type.trim().to_string();

    let activity_type = if activity_type.is_empty() {
        None
    } else {
        Some(activity_type)
    };

    // Customer name (optional)
    print!("Customer name (optional): ");
    io::stdout().flush()?;
    let mut customer = String::new();
    io::stdin().read_line(&mut customer)?;
    let customer = customer.trim().to_string();
    let customer = if customer.is_empty() {
        None
    } else {
        Some(customer)
    };

    // Work item (optional)
    print!("Work item (optional): ");
    io::stdout().flush()?;
    let mut work_item = String::new();
    io::stdin().read_line(&mut work_item)?;
    let work_item = work_item.trim().to_string();
    let work_item = if work_item.is_empty() {
        None
    } else {
        Some(work_item)
    };

    // NEW: Comment (optional)
    print!("Comment (optional): ");
    io::stdout().flush()?;
    let mut comment = String::new();
    io::stdin().read_line(&mut comment)?;
    let comment = comment.trim().to_string();
    let comment = if comment.is_empty() {
        None
    } else {
        Some(comment)
    };

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

    Ok((
        date,
        activity_type,
        customer,
        work_item,
        hours,
        days,
        comment,
    ))
}

fn validate_date_flexible(date_str: &str) -> Result<()> {
    validate_date(date_str)
}

// Keep only the functions needed for the add functionality
async fn create_items_on_monday(
    client: &MondayClient,
    actual_dates: &[chrono::NaiveDate],
    activity_type_value: u8,
    customer: &Option<String>,
    work_item: &Option<String>,
    comment: &Option<String>, // NEW: Comment parameter
    hours: Option<f64>,
    user_id: i64,
    user_name: &str,
    group_id: &str,
    verbose: bool,
) -> Result<()> {
    use serde_json::json;
    use tokio::time;

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

        // NEW: Set comment if provided
        if let Some(cmt) = comment {
            if !cmt.is_empty() {
                column_values["text"] = json!(cmt);
            }
        }

        // Set hours if provided
        if let Some(h) = hours {
            column_values["numbers__1"] = json!(h.to_string());
        }

        if verbose {
            println!(
                "\n📋 GraphQL Mutation for {} ({} of {}):",
                date_str,
                i + 1,
                actual_dates.len()
            );
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

        println!(
            "Creating item for {} ({} of {})...",
            date_str,
            i + 1,
            actual_dates.len()
        );

        match client
            .create_item_verbose(board_id, group_id, user_name, &column_values, verbose)
            .await
        {
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
        time::sleep(time::Duration::from_millis(1)).await;
    }

    println!(
        "\n🎉 Successfully created {} out of {} items",
        successful_creations,
        actual_dates.len()
    );

    if successful_creations < actual_dates.len() {
        return Err(anyhow!(
            "Some items failed to create. Check the errors above."
        ));
    }

    Ok(())
}

fn show_graphql_mutations(
    actual_dates: &[chrono::NaiveDate],
    activity_type_value: &u8,
    customer: &Option<String>,
    work_item: &Option<String>,
    comment: &Option<String>, // NEW: Comment parameter
    hours: Option<f64>,
    user_id: i64,
    user_name: &str,
    group_id: &str,
) {
    use serde_json::json;

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

        // NEW: Set comment if provided
        if let Some(cmt) = comment {
            if !cmt.is_empty() {
                column_values["text"] = json!(cmt);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_api_key() {
        assert_eq!(mask_api_key("12345678"), "1234****");
        assert_eq!(mask_api_key("1234"), "****");
        assert_eq!(mask_api_key("123"), "***");
    }

    #[test]
    fn test_normalize_date() {
        assert_eq!(normalize_date("2025-09-15"), "2025-09-15");
        assert_eq!(normalize_date("2025.09.15"), "2025-09-15");
        assert_eq!(normalize_date("2025/09/15"), "2025-09-15");
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
    }

    #[test]
    fn test_calculate_working_dates() {
        let start_date = chrono::NaiveDate::from_ymd_opt(2025, 9, 15).unwrap(); // Monday
        let dates = calculate_working_dates(start_date, 5);
        assert_eq!(dates.len(), 5);
    }

    #[test]
    fn test_get_current_year() {
        let year = utils::get_current_year();
        let current_year = chrono::Local::now().year();
        assert_eq!(year, current_year);
    }
}
