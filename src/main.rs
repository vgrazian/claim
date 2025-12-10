mod add;
mod config;
mod delete;
mod monday;
mod query;
mod selenium;
mod time;
mod utils;

use anyhow::{anyhow, Result};
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
    /// Delete a claim item by ID or by date + customer + work item
    Delete {
        /// Item ID to delete
        #[arg(short = 'x', long = "id")]
        delete_id: Option<String>,

        /// Date to filter claims (YYYY-MM-DD, YYYY.MM.DD, or YYYY/MM/DD format)
        #[arg(short = 'D', long = "date")]
        date: Option<String>,

        /// Customer name to filter by
        #[arg(short = 'c', long = "customer")]
        customer: Option<String>,

        /// Work item to filter by
        #[arg(short = 'w', long = "wi")]
        work_item: Option<String>,

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
            date,
            customer,
            work_item,
            yes,
            verbose,
        }) => {
            delete::handle_delete_command(
                &client,
                &user,
                &current_year,
                delete_id,
                date,
                customer,
                work_item,
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

// Re-export utility functions for use in other modules
pub use utils::{
    calculate_working_dates, get_year_group_id, map_activity_type_to_value,
    map_activity_value_to_name, mask_api_key, normalize_date, truncate_string, validate_date,
};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

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
