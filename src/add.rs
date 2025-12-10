use crate::monday::{Board, MondayClient, MondayUser};
use crate::{calculate_working_dates, map_activity_type_to_value, normalize_date, validate_date};
use anyhow::{anyhow, Result};
use chrono::prelude::*;
use serde_json::json;
use std::io;
use tokio::time;

pub async fn handle_add_command(
    client: &MondayClient,
    user: &MondayUser,
    current_year: &str,
    date: Option<String>,
    activity_type: Option<String>,
    customer: Option<String>,
    work_item: Option<String>,
    hours: Option<f64>,
    days: Option<f64>,
    comment: Option<String>,
    yes: bool,
    verbose: bool,
) -> Result<()> {
    let (
        final_date,
        final_activity_type,
        final_customer,
        final_work_item,
        final_hours,
        final_days,
        final_comment,
        is_interactive,
    ) = if date.is_none()
        && activity_type.is_none()
        && customer.is_none()
        && work_item.is_none()
        && hours.is_none()
        && days.is_none()
        && comment.is_none()
    {
        let (d, at, c, wi, h, d_val, cmt) = prompt_for_claim_details()?;
        (d, at, c, wi, h, d_val, cmt, true)
    } else {
        if let Some(ref d) = date {
            validate_date(d)?;
        }
        (
            date.unwrap_or_default(),
            activity_type,
            customer,
            work_item,
            hours,
            days,
            comment,
            false,
        )
    };

    let final_date = if final_date.is_empty() {
        Local::now().format("%Y-%m-%d").to_string()
    } else {
        final_date
    };

    let activity_type_str = final_activity_type.unwrap_or_else(|| "billable".to_string());
    let activity_type_value = map_activity_type_to_value(&activity_type_str);
    let days_value = final_days.unwrap_or(1.0);

    let start_date = chrono::NaiveDate::parse_from_str(&final_date, "%Y-%m-%d")?;
    let target_days = days_value as i64;
    let actual_dates = calculate_working_dates(start_date, target_days);

    println!("\n=== Adding Claim for User ===");
    println!(
        "User ID: {}, Name: {}, Email: {}",
        user.id, user.name, user.email
    );
    println!("Year: {}", current_year);
    println!("\n=== Claim Details ===");
    println!("Date: {}", final_date);
    println!(
        "Activity Type: {} (value: {})",
        activity_type_str, activity_type_value
    );
    println!(
        "Customer: {}",
        final_customer.as_deref().unwrap_or("Not specified")
    );
    println!(
        "Work Item: {}",
        final_work_item.as_deref().unwrap_or("Not specified")
    );
    println!(
        "Comment: {}",
        final_comment.as_deref().unwrap_or("Not specified")
    );
    println!(
        "Hours: {}",
        final_hours
            .map(|h| h.to_string())
            .unwrap_or_else(|| "Not specified".to_string())
    );
    println!("Days requested: {}", days_value);
    println!("Actual working days: {}", actual_dates.len());

    println!("\n📅 Dates that will be created (weekends skipped):");
    for (i, date) in actual_dates.iter().enumerate() {
        let weekday = date.format("%A");
        println!("  {}. {} ({})", i + 1, date.format("%Y-%m-%d"), weekday);
    }

    let board = client
        .query_board_verbose("6500270039", current_year, user.id, 1, verbose)
        .await?;
    let group_id = get_year_group_id(&board, current_year);

    if verbose {
        println!(
            "\n🔍 Verbose mode: Found group '{}' with ID: {}",
            current_year, group_id
        );
        show_graphql_mutations(
            &actual_dates,
            &activity_type_value,
            &final_customer,
            &final_work_item,
            &final_comment,
            final_hours,
            user.id,
            &user.name,
            &group_id,
        );
    } else {
        println!("\nFound group '{}' with ID: {}", current_year, group_id);
    }

    if !yes {
        println!(
            "\n🚀 Ready to create {} item(s) on Monday.com",
            actual_dates.len()
        );
        println!("Do you want to proceed? (y/N)");

        let mut confirmation = String::new();
        io::stdin().read_line(&mut confirmation)?;

        if confirmation.trim().to_lowercase() != "y" {
            println!("Operation cancelled.");
            return Ok(());
        }
    } else {
        println!(
            "\n🚀 Creating {} item(s) on Monday.com (skipping confirmation)",
            actual_dates.len()
        );
    }

    create_items_on_monday(
        client,
        &actual_dates,
        activity_type_value,
        &final_customer,
        &final_work_item,
        &final_comment,
        final_hours,
        user.id,
        &user.name,
        &group_id,
        verbose,
    )
    .await?;

    if is_interactive {
        show_equivalent_command(
            &final_date,
            &activity_type_str,
            &final_customer,
            &final_work_item,
            &final_comment,
            final_hours,
            days_value,
            yes,
            verbose,
        );
    }

    Ok(())
}

async fn create_items_on_monday(
    client: &MondayClient,
    actual_dates: &[NaiveDate],
    activity_type_value: u8,
    customer: &Option<String>,
    work_item: &Option<String>,
    comment: &Option<String>,
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

        // FIXED: Set comment using the correct column ID "text2__1"
        if let Some(cmt) = comment {
            if !cmt.is_empty() {
                column_values["text2__1"] = json!(cmt);
                if verbose {
                    println!("   Setting comment in column 'text2__1': '{}'", cmt);
                }
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
        time::sleep(time::Duration::from_millis(200)).await;
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
    actual_dates: &[NaiveDate],
    activity_type_value: &u8,
    customer: &Option<String>,
    work_item: &Option<String>,
    comment: &Option<String>,
    hours: Option<f64>,
    user_id: i64,
    user_name: &str,
    group_id: &str,
) {
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

        // FIXED: Set comment using the correct column ID "text2__1"
        if let Some(cmt) = comment {
            if !cmt.is_empty() {
                column_values["text2__1"] = json!(cmt);
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

fn get_year_group_id(board: &Board, year: &str) -> String {
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

fn show_equivalent_command(
    date: &str,
    activity_type: &str,
    customer: &Option<String>,
    work_item: &Option<String>,
    comment: &Option<String>,
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

    // Include comment if provided
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
    Option<String>,
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

    // Activity type (optional, defaults to billable) - IMPROVED: Show available options
    println!("\nAvailable activity types:");
    println!(" 0 - vacation");
    println!(" 1 - billable (default)");
    println!(" 2 - holding");
    println!(" 3 - education");
    println!(" 4 - work_reduction");
    println!(" 5 - tbd");
    println!(" 6 - holiday");
    println!(" 7 - presales");
    println!(" 8 - illness");
    println!(" 9 - paid_not_worked");
    println!("10 - intellectual_capital");
    println!("11 - business_development");
    println!("12 - overhead");
    print!("\nActivity type (enter number or name, optional - default: billable): ");
    io::stdout().flush()?;
    let mut activity_type = String::new();
    io::stdin().read_line(&mut activity_type)?;
    let activity_type = activity_type.trim().to_string();

    let activity_type = if activity_type.is_empty() {
        None
    } else {
        // Handle numeric input
        if let Ok(num) = activity_type.parse::<u8>() {
            match num {
                0 => Some("vacation".to_string()),
                1 => Some("billable".to_string()),
                2 => Some("holding".to_string()),
                3 => Some("education".to_string()),
                4 => Some("work_reduction".to_string()),
                5 => Some("tbd".to_string()),
                6 => Some("holiday".to_string()),
                7 => Some("presales".to_string()),
                8 => Some("illness".to_string()),
                9 => Some("paid_not_worked".to_string()),
                10 => Some("intellectual_capital".to_string()),
                11 => Some("business_development".to_string()),
                12 => Some("overhead".to_string()),
                _ => {
                    println!("Invalid activity type number. Using default 'billable'.");
                    Some("billable".to_string())
                }
            }
        } else {
            // Handle text input with case insensitivity and flexible formatting
            let normalized_type = normalize_activity_type_input(&activity_type);
            match normalized_type.as_str() {
                "vacation" | "0" => Some("vacation".to_string()),
                "billable" | "1" => Some("billable".to_string()),
                "holding" | "2" => Some("holding".to_string()),
                "education" | "3" => Some("education".to_string()),
                "work_reduction" | "4" => Some("work_reduction".to_string()),
                "tbd" | "5" => Some("tbd".to_string()),
                "holiday" | "6" => Some("holiday".to_string()),
                "presales" | "7" => Some("presales".to_string()),
                "illness" | "8" => Some("illness".to_string()),
                "paid_not_worked" | "9" => Some("paid_not_worked".to_string()),
                "intellectual_capital" | "10" => Some("intellectual_capital".to_string()),
                "business_development" | "11" => Some("business_development".to_string()),
                "overhead" | "12" => Some("overhead".to_string()),
                _ => {
                    println!(
                        "❌ Error: Unknown activity type '{}'. Please use a valid number or name.",
                        activity_type
                    );
                    println!("Valid options: vacation, billable, holding, education, work_reduction, tbd, holiday, presales, illness, paid_not_worked, intellectual_capital, business_development, overhead");
                    return Err(anyhow!("Unknown activity type: {}", activity_type));
                }
            }
        }
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

    // Comment (optional)
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

// Helper function to normalize activity type input
fn normalize_activity_type_input(input: &str) -> String {
    let normalized = input.to_lowercase().replace(' ', "_").replace('-', "_");

    // Handle common variations
    match normalized.as_str() {
        "work_reduction" | "workreduction" => "work_reduction".to_string(),
        "paid_not_worked" | "paidnotworked" => "paid_not_worked".to_string(),
        "intellectual_capital" | "intellectualcapital" => "intellectual_capital".to_string(),
        "business_development" | "businessdevelopment" => "business_development".to_string(),
        "overhead" | "over_head" => "overhead".to_string(),
        _ => normalized,
    }
}

fn validate_date_flexible(date_str: &str) -> Result<()> {
    validate_date(date_str)
}

//  tests ...

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monday::{Board, Group};

    #[test]
    fn test_get_year_group_id() {
        let board = Board {
            groups: Some(vec![
                Group {
                    id: "group_2024".to_string(),
                    title: "2024".to_string(),
                    items_page: None,
                },
                Group {
                    id: "group_2025".to_string(),
                    title: "2025".to_string(),
                    items_page: None,
                },
            ]),
        };

        assert_eq!(get_year_group_id(&board, "2025"), "group_2025");
        assert_eq!(get_year_group_id(&board, "2024"), "group_2024");
        assert_eq!(get_year_group_id(&board, "2023"), "new_group_mkkbbd2q"); // fallback
    }

    #[test]
    fn test_get_year_group_id_no_groups() {
        let board = Board { groups: None };

        assert_eq!(get_year_group_id(&board, "2025"), "new_group_mkkbbd2q");
    }

    #[test]
    fn test_show_equivalent_command() {
        // Test that the function doesn't panic with various inputs
        show_equivalent_command(
            "2025-09-15",
            "billable",
            &Some("Customer".to_string()),
            &Some("WorkItem".to_string()),
            &Some("Test comment".to_string()), // NEW: Test with comment
            Some(8.0),
            1.0,
            false,
            false,
        );

        show_equivalent_command(
            "2025-09-15",
            "vacation",
            &None,
            &None,
            &None, // NEW: Test without comment
            None,
            5.0,
            true,
            true,
        );

        // Should not panic with empty values
        show_equivalent_command(
            "2025-09-15",
            "billable",
            &None,
            &None,
            &None,
            None,
            1.0,
            false,
            false,
        );
    }

    #[test]
    fn test_validate_date_flexible() {
        assert!(validate_date_flexible("2025-09-15").is_ok());
        assert!(validate_date_flexible("2025.09.15").is_ok());
        assert!(validate_date_flexible("2025/09/15").is_ok());
        assert!(validate_date_flexible("invalid-date").is_err());
    }

    #[test]
    fn test_prompt_functions_do_not_panic() {
        // These are integration-style tests that verify the functions don't panic
        // We can't easily test the actual IO, but we can test that the functions are properly defined

        // Test that the function signatures are correct
        let result = validate_date_flexible("2025-09-15");
        assert!(result.is_ok());

        // Test that get_year_group_id returns a string
        let board = Board { groups: None };
        let group_id = get_year_group_id(&board, "2025");
        assert!(!group_id.is_empty());
    }

    #[test]
    fn test_command_generation_edge_cases() {
        // Test with special characters that might need escaping
        show_equivalent_command(
            "2025-09-15",
            "billable",
            &Some("Customer with spaces".to_string()),
            &Some("Work/Item".to_string()),
            &Some("Comment with \"quotes\"".to_string()), // NEW: Test comment with special chars
            Some(7.5),
            2.5,
            true,
            false,
        );

        // Test with very long values
        show_equivalent_command(
            "2025-09-15",
            "billable",
            &Some("A".repeat(100)),
            &Some("B".repeat(100)),
            &Some("C".repeat(100)), // NEW: Test long comment
            Some(999.99),
            99.0,
            false,
            true,
        );
    }

    #[test]
    fn test_date_calculation_integration() {
        // Test that the date calculation works correctly
        let start_date = NaiveDate::from_ymd_opt(2025, 9, 15).unwrap(); // Monday
        let dates = calculate_working_dates(start_date, 5);

        // Should get 5 weekdays
        assert_eq!(dates.len(), 5);
        assert_eq!(dates[0].weekday(), Weekday::Mon);
        assert_eq!(dates[1].weekday(), Weekday::Tue);
        assert_eq!(dates[2].weekday(), Weekday::Wed);
        assert_eq!(dates[3].weekday(), Weekday::Thu);
        assert_eq!(dates[4].weekday(), Weekday::Fri);
    }

    #[test]
    fn test_activity_type_mapping() {
        assert_eq!(map_activity_type_to_value("billable"), 1);
        assert_eq!(map_activity_type_to_value("vacation"), 0);
        assert_eq!(map_activity_type_to_value("holding"), 2);
        assert_eq!(map_activity_type_to_value("unknown"), 1); // default
    }

    #[test]
    fn test_json_creation() {
        // Test that JSON creation doesn't panic
        let column_values = json!({
            "person": {
                "personsAndTeams": [
                    {
                        "id": 123,
                        "kind": "person"
                    }
                ]
            },
            "date4": {
                "date": "2025-09-15"
            },
            "status": {
                "index": 1
            },
            "text": "Test comment"  // NEW: Test comment field
        });

        assert!(!column_values.to_string().is_empty());
    }

    #[test]
    fn test_normalize_activity_type_input() {
        // Test case insensitivity
        assert_eq!(normalize_activity_type_input("VACATION"), "vacation");
        assert_eq!(normalize_activity_type_input("Billable"), "billable");

        // Test space to underscore conversion
        assert_eq!(
            normalize_activity_type_input("work reduction"),
            "work_reduction"
        );
        assert_eq!(
            normalize_activity_type_input("business development"),
            "business_development"
        );

        // Test hyphen to underscore conversion
        assert_eq!(
            normalize_activity_type_input("work-reduction"),
            "work_reduction"
        );
        assert_eq!(
            normalize_activity_type_input("business-development"),
            "business_development"
        );

        // Test mixed cases
        assert_eq!(
            normalize_activity_type_input("Work-Reduction"),
            "work_reduction"
        );
        assert_eq!(
            normalize_activity_type_input("Business Development"),
            "business_development"
        );

        // Test already normalized inputs
        assert_eq!(normalize_activity_type_input("vacation"), "vacation");
        assert_eq!(
            normalize_activity_type_input("work_reduction"),
            "work_reduction"
        );

        // Test the corrected activity types
        assert_eq!(
            normalize_activity_type_input("paid not worked"),
            "paid_not_worked"
        );
        assert_eq!(
            normalize_activity_type_input("paid-not-worked"),
            "paid_not_worked"
        );
        assert_eq!(
            normalize_activity_type_input("intellectual capital"),
            "intellectual_capital"
        );
        assert_eq!(
            normalize_activity_type_input("intellectual-capital"),
            "intellectual_capital"
        );
        assert_eq!(normalize_activity_type_input("over head"), "overhead");
        assert_eq!(normalize_activity_type_input("over-head"), "overhead");
    }
}
