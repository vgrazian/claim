mod add;
mod config;
mod monday;
mod query;

use anyhow::{anyhow, Result};
use chrono::prelude::*;
use clap::{Parser, Subcommand};
use config::Config;
use monday::{MondayClient, MondayUser};
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
    let current_year = get_current_year().to_string();

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
            verbose,
        }) => {
            query::handle_query_command(&client, &user, limit, date, days, verbose).await?;
        }
        Some(Commands::Add {
            date,
            activity_type,
            customer,
            work_item,
            hours,
            days,
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
                yes,
                verbose,
            )
            .await?;
        }
        None => {
            // Default action when no command is provided
            println!("No command specified. Use --help for available commands.");
        }
    }

    Ok(())
}

// Common helper functions - these are used by both query.rs and add.rs
pub fn get_current_year() -> i32 {
    let now = Local::now();
    now.year()
}

pub fn mask_api_key(api_key: &str) -> String {
    if api_key.len() <= 4 {
        "*".repeat(api_key.len())
    } else {
        let visible_part = &api_key[..4];
        let masked_part = "*".repeat(api_key.len() - 4);
        format!("{}{}", visible_part, masked_part)
    }
}

pub fn validate_date(date_str: &str) -> Result<()> {
    // Try multiple date formats
    let formats = ["%Y-%m-%d", "%Y.%m.%d", "%Y/%m/%d"];

    for format in &formats {
        if chrono::NaiveDate::parse_from_str(date_str, format).is_ok() {
            return Ok(());
        }
    }

    Err(anyhow!(
        "Invalid date format: {}. Please use YYYY-MM-DD, YYYY.MM.DD, or YYYY/MM/DD format.",
        date_str
    ))
}

pub fn normalize_date(date_str: &str) -> String {
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

pub fn calculate_working_dates(start_date: NaiveDate, target_days: i64) -> Vec<NaiveDate> {
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

pub fn map_activity_type_to_value(activity_type: &str) -> u8 {
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
            println!(
                "Warning: Unknown activity type '{}', defaulting to billable (1)",
                activity_type
            );
            1 // Default to billable for unknown types
        }
    }
}

pub fn map_activity_value_to_name(value: u8) -> String {
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

pub fn truncate_string(s: &str, max_length: usize) -> String {
    if s.len() <= max_length {
        s.to_string()
    } else {
        format!("{}...", &s[..max_length.saturating_sub(3)])
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
        assert_eq!(mask_api_key(""), "");
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
        assert_eq!(map_activity_type_to_value("unknown"), 1);
    }

    #[test]
    fn test_map_activity_value_to_name() {
        assert_eq!(map_activity_value_to_name(1), "billable");
        assert_eq!(map_activity_value_to_name(0), "vacation");
        assert_eq!(map_activity_value_to_name(99), "unknown(99)");
    }

    #[test]
    fn test_calculate_working_dates() {
        let start_date = NaiveDate::from_ymd_opt(2025, 9, 15).unwrap(); // Monday
        let dates = calculate_working_dates(start_date, 5);
        assert_eq!(dates.len(), 5);
        assert_eq!(dates[0].weekday(), Weekday::Mon);
        assert_eq!(dates[4].weekday(), Weekday::Fri);
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("short", 10), "short");
        assert_eq!(truncate_string("very long string", 10), "very lo...");
    }

    #[test]
    fn test_get_current_year() {
        let year = get_current_year();
        let current_year = Local::now().year();
        assert_eq!(year, current_year);
    }
}
