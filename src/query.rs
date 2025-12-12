use crate::monday::{Item, MondayClient, MondayUser};
use crate::{
    calculate_working_dates, get_year_group_id, map_activity_value_to_name, normalize_date,
    truncate_string, validate_date,
};
use anyhow::Result;
use chrono::prelude::*;
use rand::seq::SliceRandom;
use std::io::{self, Write};
use std::time::Duration;
use tokio::task;

pub async fn handle_query_command(
    client: &MondayClient,
    user: &MondayUser,
    limit: usize,
    date: Option<String>,
    days: usize,
    customer: Option<String>,  // NEW: Customer filter
    work_item: Option<String>, // NEW: Work item filter
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
        if let Some(_start_date_val) = start_date {
            if target_days > 1 {
                let end_date = date_range
                    .last()
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_default();
                println!(
                    "Querying board {} for user '{}' with date range: {} to {} ({} working days)...",
                    board_id,
                    user.name,
                    _start_date_val.format("%Y-%m-%d"),
                    end_date,
                    target_days
                );
            } else {
                println!(
                    "Querying board {} for user '{}' with date filter: {}...",
                    board_id,
                    user.name,
                    _start_date_val.format("%Y-%m-%d")
                );
            }
        } else {
            println!("Querying board {} for user '{}'...", board_id, user.name);
        }

        // NEW: Show customer and work item filters if provided
        if let Some(ref c) = customer {
            println!("Customer filter: {}", c);
        }
        if let Some(ref wi) = work_item {
            println!("Work item filter: {}", wi);
        }
    } else {
        // Show brief info even in non-verbose mode
        if let Some(_start_date_val) = start_date {
            if target_days > 1 {
                let end_date = date_range
                    .last()
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_default();
                println!(
                    "Querying date range: {} to {} ({} days)...",
                    _start_date_val.format("%Y-%m-%d"),
                    end_date,
                    target_days
                );
            } else {
                println!("Querying date: {}...", _start_date_val.format("%Y-%m-%d"));
            }
        }

        // NEW: Show filters briefly
        if customer.is_some() || work_item.is_some() {
            let mut filters = Vec::new();
            if let Some(c) = &customer {
                filters.push(format!("customer: {}", c));
            }
            if let Some(wi) = &work_item {
                filters.push(format!("work item: {}", wi));
            }
            println!("Filters: {}", filters.join(", "));
        }
    }

    // Start the dog walking animation
    let animation_handle = if !verbose && start_date.is_some() {
        Some(start_walking_dog_animation())
    } else {
        None
    };

    // Get the current year's group ID
    let current_year = get_current_year().to_string();
    let board = client.get_board_with_groups(board_id, verbose).await?;
    let group_id = get_year_group_id(&board, &current_year);

    if verbose {
        println!("Using group ID: {} for year: {}", group_id, current_year);
    }

    // Convert date_range to strings for the query
    let date_strings: Vec<String> = date_range
        .iter()
        .map(|d| d.format("%Y-%m-%d").to_string())
        .collect();

    // Use server-side filtering to get items
    let filtered_items = client
        .query_items_with_filters(
            board_id,
            &group_id,
            user.id,
            &date_strings,
            limit,
            verbose,
        )
        .await?;

    if verbose {
        println!("\n=== Server-side filtered items: {} ===", filtered_items.len());
    }

    // Stop the animation if it's running
    if let Some(handle) = animation_handle {
        stop_walking_dog_animation(handle).await;
    }

    // Determine if we have exact matches
    let has_exact_matches = if !date_range.is_empty() {
        filtered_items
            .iter()
            .any(|item| is_item_matching_date_range(item, &date_range))
    } else {
        true
    };

    // For display purposes
    let limited_items: Vec<Item> = filtered_items.iter().take(limit).cloned().collect();
    let filtered_items_len = filtered_items.len();

    // Display the results
    if !filtered_items.is_empty() {
        if let Some(_start_date_val) = start_date {
            if target_days > 1 {
                // Multi-day query - show simplified table
                display_simplified_table(
                    &filtered_items,
                    &date_range,
                    &user.name,
                    verbose,
                    has_exact_matches,
                    false,
                );
            } else {
                // Single day query - show detailed format
                display_detailed_items(
                    &limited_items,
                    start_date,
                    &user.name,
                    filtered_items_len,
                    limit,
                    has_exact_matches,
                    &customer,
                    &work_item,
                );
            }
        } else {
            // No date filter - show detailed format
            display_detailed_items(
                &limited_items,
                None,
                &user.name,
                filtered_items_len,
                limit,
                true,
                &customer,
                &work_item,
            );
        }
    } else {
        println!("\nNo items found for user '{}'", user.name);

        // Show applied filters in the "no results" message
        let mut filter_info = Vec::new();
        if let Some(_start_date_val) = start_date {
            if target_days > 1 {
                let end_date = date_range
                    .last()
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_default();
                filter_info.push(format!(
                    "date range: {} to {} ({} working days)",
                    _start_date_val.format("%Y-%m-%d"),
                    end_date,
                    target_days
                ));
            } else {
                filter_info.push(format!("date: {}", _start_date_val.format("%Y-%m-%d")));
                }
        }
        if let Some(c) = &customer {
            filter_info.push(format!("customer: {}", c));
        }
        if let Some(wi) = &work_item {
            filter_info.push(format!("work item: {}", wi));
        }

        if !filter_info.is_empty() {
            println!("Filters: {}", filter_info.join(", "));
        }
    }

    // Show final message based on results
    if let Some(query_date) = start_date {
        if target_days > 1 {
            let end_date = date_range
                .last()
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default();

            if has_exact_matches {
                println!(
                    "\n✅ Found {} total items matching date range: {} to {}",
                    filtered_items_len,
                    query_date.format("%Y-%m-%d"),
                    end_date
                );
            } else if filtered_items_len > 0 {
                println!(
                    "\n⚠️  Showing {} items from date range: {} to {}",
                    filtered_items_len,
                    query_date.format("%Y-%m-%d"),
                    end_date
                );
            }
        } else {
            if has_exact_matches {
                println!(
                    "\n✅ Found {} total items matching date filter: {}",
                    filtered_items_len,
                    query_date.format("%Y-%m-%d")
                );
            } else if filtered_items_len > 0 {
                println!(
                    "\n⚠️  Showing {} items near date: {}",
                    filtered_items_len,
                    query_date.format("%Y-%m-%d")
                );
            }
        }
    }

    // Show final summary message
    if let Some(query_date) = start_date {
        if target_days > 1 {
            let end_date = date_range
                .last()
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default();

            if has_exact_matches && filtered_items_len > 0 {
                println!(
                    "\n✅ Found {} total items matching date range: {} to {}",
                    filtered_items_len,
                    query_date.format("%Y-%m-%d"),
                    end_date
                );
            }
        } else {
            if has_exact_matches && filtered_items_len > 0 {
                println!(
                    "\n✅ Found {} total items matching date filter: {}",
                    filtered_items_len,
                    query_date.format("%Y-%m-%d")
                );
            }
        }
    }

    Ok(())
}

// Improved walking dog animation - simpler and more reliable
fn start_walking_dog_animation() -> tokio::task::JoinHandle<()> {
    task::spawn(async move {
        let dog_frames = [
            "🐕‍🦺       ",
            " 🐕‍🦺      ",
            "  🐕‍🦺     ",
            "   🐕‍🦺    ",
            "    🐕‍🦺   ",
            "     🐕‍🦺  ",
            "      🐕‍🦺 ",
            "       🐕‍🦺",
            "      🐕‍🦺 ",
            "     🐕‍🦺  ",
            "    🐕‍🦺   ",
            "   🐕‍🦺    ",
            "  🐕‍🦺     ",
            " 🐕‍🦺      ",
        ];
        let messages = [
            "Searching your claims... perfect time to walk the dog! 🐕",
            "Fetching data... your dog would love some fresh air! 🦮",
            "Looking through entries... why not take the dog out? 🐩",
        ];

        let message = {
            let mut rng = rand::thread_rng();
            messages.choose(&mut rng).unwrap_or(&messages[0])
        };

        println!("\n{}", message);

        for i in 0.. {
            print!("\rPlease wait {}", dog_frames[i % dog_frames.len()]);
            io::stdout().flush().unwrap();

            // Check if we should stop (every 100ms)
            if let Ok(_) =
                tokio::time::timeout(Duration::from_millis(100), std::future::pending::<()>()).await
            {
                break;
            }
        }

        // Clear the animation line
        print!("\r{}", " ".repeat(50));
        print!("\r");
        io::stdout().flush().unwrap();
    })
}

// Function to stop the animation
async fn stop_walking_dog_animation(handle: tokio::task::JoinHandle<()>) {
    handle.abort();
    let _ = handle.await; // Wait for it to finish

    // Small delay to ensure the terminal is cleared
    tokio::time::sleep(Duration::from_millis(50)).await;
}

// Helper function to get year group ID

// Fixed: Strict date matching function
fn is_item_matching_date(item: &Item, target_date: &str) -> bool {
    for col in &item.column_values {
        if let Some(col_id) = &col.id {
            if col_id == "date4" {
                // Parse the date column value to check if it matches the target date exactly
                if let Some(value) = &col.value {
                    if let Ok(parsed_value) = serde_json::from_str::<serde_json::Value>(value) {
                        if let Some(date_obj) = parsed_value.get("date") {
                            if let Some(date_str) = date_obj.as_str() {
                                // Compare the date part exactly (YYYY-MM-DD)
                                if date_str == target_date {
                                    return true;
                                }
                                // Also check if it starts with the target date (in case of datetime strings)
                                if date_str.starts_with(target_date) {
                                    // But only if it's exactly the date part (not a partial match)
                                    if date_str.len() >= target_date.len() {
                                        let date_part = &date_str[..target_date.len()];
                                        if date_part == target_date {
                                            return true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                // Also check the text field as fallback
                if let Some(text) = &col.text {
                    if text == target_date {
                        return true;
                    }
                    if text.starts_with(target_date) {
                        if text.len() >= target_date.len() {
                            let date_part = &text[..target_date.len()];
                            if date_part == target_date {
                                return true;
                            }
                        }
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
        "text__1" => "Customer",
        "text8__1" => "Work Item",
        "text2__1" => "Comment", // FIXED: Map text2__1 to Comment
        "numbers__1" => "Hours",
        "hours" => "Hours",
        "days" => "Days",
        "activity_type" => "Activity Type",
        "customer" => "Customer",
        "work_item" => "Work Item",
        "text" => "Text",
        _ => column_id,
    }
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
                            // For person columns, try to extract name
                            if let Some(persons) = parsed_value.get("personsAndTeams") {
                                if let Some(persons_array) = persons.as_array() {
                                    if let Some(first_person) = persons_array.first() {
                                        if let Some(name) =
                                            first_person.get("name").and_then(|n| n.as_str())
                                        {
                                            return name.to_string();
                                        }
                                    }
                                }
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

// NEW: Helper function to extract comment value from the correct column
fn extract_comment_value(item: &Item) -> String {
    for col in &item.column_values {
        if let Some(col_id) = &col.id {
            // FIXED: Use the correct comment column ID "text2__1"
            if col_id == "text2__1" {
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

// Display simplified table for multi-day queries - UPDATED to show comments
fn display_simplified_table(
    items: &[Item],
    date_range: &[NaiveDate],
    user_name: &str,
    verbose: bool,
    has_exact_matches: bool,
    has_filters: bool,
) {
    println!("\n=== CLAIMS SUMMARY for User {} ===", user_name);

    let start_date = date_range
        .first()
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default();
    let end_date = date_range
        .last()
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default();

    println!("Date Range: {} to {}", start_date, end_date);

    if !has_exact_matches && !items.is_empty() {
        println!("💡 Showing items from nearby dates (no exact matches found in range)");
    }

    if verbose {
        println!(
            "Processing {} items across {} dates",
            items.len(),
            date_range.len()
        );
    }

    // UPDATED: Create a table header with Comment column
    println!(
        "\n{:<12} {:<12} {:<20} {:<15} {:<6} {:<20}",
        "Date", "Status", "Customer", "Work Item", "Hours", "Comment"
    );
    println!("{}", "-".repeat(90));

    // Group items by date using a HashMap with exact date matching
    let mut items_by_date: std::collections::HashMap<String, Vec<&Item>> =
        std::collections::HashMap::new();

    for item in items {
        if let Some(item_date_str) = extract_item_date(item) {
            if let Ok(item_date) = chrono::NaiveDate::parse_from_str(&item_date_str, "%Y-%m-%d") {
                // Only include items that exactly match dates in the range
                if date_range.contains(&item_date) {
                    items_by_date
                        .entry(item_date_str)
                        .or_insert_with(Vec::new)
                        .push(item);
                }
            }
        }
    }

    // Display items in date range order
    let mut total_hours: f64 = 0.0;
    let mut displayed_items = 0;
    let mut displayed_dates_count = 0;

    for date in date_range {
        let date_str = date.format("%Y-%m-%d").to_string();

        if let Some(date_items) = items_by_date.get(&date_str) {
            // Always show dates that have items
            displayed_dates_count += 1;
            displayed_items += date_items.len();
            for item in date_items {
                let status = extract_status_value(item);
                let customer = extract_column_value(item, "text__1");
                let work_item = extract_column_value(item, "text8__1");
                let hours_str = extract_column_value(item, "numbers__1");
                let comment = extract_comment_value(item); // FIXED: Extract comment from correct column
                let hours = hours_str.parse::<f64>().unwrap_or(0.0);
                total_hours += hours;

                println!(
                    "{:<12} {:<12} {:<20} {:<15} {:<6} {:<20}",
                    date_str,
                    truncate_string(&status, 10),
                    truncate_string(&customer, 18),
                    truncate_string(&work_item, 13),
                    hours_str,
                    truncate_string(&comment, 18)
                );
            }
        } else if !has_filters {
            // Only show empty rows when no filters are active
            displayed_dates_count += 1;
            println!(
                "{:<12} {:<12} {:<20} {:<15} {:<6} {:<20}",
                date_str, "-", "-", "-", "-", "-"
            );
        }
    }

    println!("{}", "-".repeat(90));
    println!(
        "{:<12} {:<12} {:<20} {:<15} {:<6.1} {:<20}",
        "TOTAL", "", "", "", total_hours, ""
    );

    if has_filters {
        println!(
            "\nFound {} items matching filters across {} days",
            displayed_items, displayed_dates_count
        );
    } else {
        println!(
            "\nFound {} items across {} days",
            displayed_items,
            date_range.len()
        );
    }

    // Show message if we have items but they don't match the exact date range
    if items.len() > 0 && displayed_items == 0 {
        // Find the closest future date
        let mut future_dates: Vec<NaiveDate> = items
            .iter()
            .filter_map(|item| extract_item_date(item))
            .filter_map(|date_str| chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").ok())
            .filter(|item_date| *item_date > *date_range.last().unwrap())
            .collect();

        future_dates.sort();
        future_dates.dedup();

        if let Some(next_date) = future_dates.first() {
            let days_diff = (*next_date - *date_range.last().unwrap()).num_days();
            let day_word = if days_diff == 1 { "day" } else { "days" };
            println!(
                "\n💡 Next available entry: {} ({} {} later)",
                next_date.format("%Y-%m-%d"),
                days_diff,
                day_word
            );
        }
    }
}

// Helper function to display detailed items (original format) - UPDATED to show comments
fn display_detailed_items(
    items: &[Item],
    filter_date: Option<NaiveDate>,
    user_name: &str,
    filtered_items_len: usize,
    limit: usize,
    has_exact_matches: bool,
    customer_filter: &Option<String>,
    work_item_filter: &Option<String>,
) {
    println!("\n=== FILTERED ITEMS for User {} ===", user_name);

    if let Some(date) = filter_date {
        println!("Date filter: {}", date.format("%Y-%m-%d"));

        if !has_exact_matches && !items.is_empty() {
            // Find the next available date after the filter date
            let mut future_dates: Vec<NaiveDate> = items
                .iter()
                .filter_map(|item| extract_item_date(item))
                .filter_map(|date_str| {
                    chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").ok()
                })
                .filter(|item_date| *item_date > date)
                .collect();

            future_dates.sort();
            future_dates.dedup();

            if let Some(next_date) = future_dates.first() {
                let days_diff = (*next_date - date).num_days();
                let day_word = if days_diff == 1 { "day" } else { "days" };

                println!(
                    "⚠️  No entries found for {}. Next available date: {} ({} {} later)",
                    date.format("%Y-%m-%d"),
                    next_date.format("%Y-%m-%d"),
                    days_diff,
                    day_word
                );
            }
        }
    }

    // Show applied filters
    if customer_filter.is_some() || work_item_filter.is_some() {
        let mut filters = Vec::new();
        if let Some(c) = customer_filter {
            filters.push(format!("customer: {}", c));
        }
        if let Some(wi) = work_item_filter {
            filters.push(format!("work item: {}", wi));
        }
        println!("Applied filters: {}", filters.join(", "));
    }

    if !has_exact_matches && !items.is_empty() {
        println!("💡 Showing items from nearby dates:");
    }

    println!("Found {} items for user {}:", filtered_items_len, user_name);

    for (index, item) in items.iter().enumerate() {
        let item_name = item.name.as_deref().unwrap_or("Unnamed");
        let item_id = item.id.as_deref().unwrap_or("Unknown");
        println!("\n{}. {} (ID: {})", index + 1, item_name, item_id);

        if !item.column_values.is_empty() {
            println!("   Columns:");
            let max_title_len = item
                .column_values
                .iter()
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
                            println!(
                                "     {:<width$} : {}",
                                column_title,
                                value,
                                width = max_title_len
                            );
                        }
                    } else if let Some(text) = &col.text {
                        if !text.is_empty() && text != "null" {
                            println!(
                                "     {:<width$} : {}",
                                column_title,
                                text,
                                width = max_title_len
                            );
                        }
                    }
                }
            }
        } else {
            println!("   No column values available");
        }
    }

    if filtered_items_len > limit {
        println!(
            "\n... and {} more items (showing first {} items)",
            filtered_items_len - limit,
            limit
        );
    }
}

// Fixed: Improved date extraction function
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
                                    // Extract just the date part (YYYY-MM-DD)
                                    if date_str.len() >= 10 {
                                        let date_part = &date_str[..10];
                                        // Validate it's a proper date format
                                        if let Ok(naive_date) =
                                            chrono::NaiveDate::parse_from_str(date_part, "%Y-%m-%d")
                                        {
                                            return Some(naive_date.format("%Y-%m-%d").to_string());
                                        }
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
                        // Extract just the date part
                        if text.len() >= 10 {
                            let date_part = &text[..10];
                            if let Ok(naive_date) =
                                chrono::NaiveDate::parse_from_str(date_part, "%Y-%m-%d")
                            {
                                return Some(naive_date.format("%Y-%m-%d").to_string());
                            }
                        }
                        return Some(text.to_string());
                    }
                }
            }
        }
    }
    None
}

// Helper function to get current year
fn get_current_year() -> i32 {
    Local::now().year()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monday::{ColumnValue, Item};

    fn create_test_item_with_date(date: &str) -> Item {
        let mut item = Item::default();
        let mut date_column = ColumnValue::default();
        date_column.id = Some("date4".to_string());
        date_column.value = Some(format!(r#"{{"date": "{}"}}"#, date));
        item.column_values.push(date_column);
        item
    }

    #[test]
    fn test_is_item_matching_date() {
        let item = create_test_item_with_date("2025-09-15");
        assert!(is_item_matching_date(&item, "2025-09-15"));
        assert!(!is_item_matching_date(&item, "2025-09-16"));

        // Test with datetime string
        let mut item_with_time = Item::default();
        let mut date_column = ColumnValue::default();
        date_column.id = Some("date4".to_string());
        date_column.value = Some(r#"{"date": "2025-09-15T00:00:00Z"}"#.to_string());
        item_with_time.column_values.push(date_column);
        assert!(is_item_matching_date(&item_with_time, "2025-09-15"));
    }

    #[test]
    fn test_is_item_matching_date_with_text() {
        let mut item = Item::default();
        let mut date_column = ColumnValue::default();
        date_column.id = Some("date4".to_string());
        date_column.text = Some("2025-09-15".to_string());
        item.column_values.push(date_column);

        assert!(is_item_matching_date(&item, "2025-09-15"));
        assert!(!is_item_matching_date(&item, "2025-09-16"));
    }

    #[test]
    fn test_is_item_matching_date_range() {
        let item = create_test_item_with_date("2025-09-15");
        let date_range = vec![
            NaiveDate::from_ymd_opt(2025, 9, 14).unwrap(),
            NaiveDate::from_ymd_opt(2025, 9, 15).unwrap(),
            NaiveDate::from_ymd_opt(2025, 9, 16).unwrap(),
        ];

        assert!(is_item_matching_date_range(&item, &date_range));

        let different_range = vec![
            NaiveDate::from_ymd_opt(2025, 9, 16).unwrap(),
            NaiveDate::from_ymd_opt(2025, 9, 17).unwrap(),
        ];

        assert!(!is_item_matching_date_range(&item, &different_range));
    }

    #[test]
    fn test_extract_item_date() {
        let item = create_test_item_with_date("2025-09-15");
        let extracted_date = extract_item_date(&item);
        assert_eq!(extracted_date, Some("2025-09-15".to_string()));

        // Test with datetime string
        let mut item_with_time = Item::default();
        let mut date_column = ColumnValue::default();
        date_column.id = Some("date4".to_string());
        date_column.value = Some(r#"{"date": "2025-09-15T12:30:45Z"}"#.to_string());
        item_with_time.column_values.push(date_column);
        let extracted_date = extract_item_date(&item_with_time);
        assert_eq!(extracted_date, Some("2025-09-15".to_string()));
    }

    #[test]
    fn test_extract_column_value() {
        let mut item = Item::default();
        let mut text_column = ColumnValue::default();
        text_column.id = Some("text__1".to_string());
        text_column.value = Some("Test Customer".to_string());
        item.column_values.push(text_column);

        let extracted_value = extract_column_value(&item, "text__1");
        assert_eq!(extracted_value, "Test Customer");
    }

    #[test]
    fn test_extract_comment_value() {
        let mut item = Item::default();
        let mut comment_column = ColumnValue::default();
        comment_column.id = Some("text2__1".to_string());
        comment_column.value = Some("Test comment".to_string());
        item.column_values.push(comment_column);

        let extracted_comment = extract_comment_value(&item);
        assert_eq!(extracted_comment, "Test comment");
    }

    #[test]
    fn test_extract_status_value() {
        let mut item = Item::default();
        let mut status_column = ColumnValue::default();
        status_column.id = Some("status".to_string());
        status_column.value = Some(r#"{"index": 1}"#.to_string());
        item.column_values.push(status_column);

        let extracted_status = extract_status_value(&item);
        assert_eq!(extracted_status, "billable");
    }

    #[test]
    fn test_map_column_title() {
        assert_eq!(map_column_title("date4"), "Date");
        assert_eq!(map_column_title("person"), "Person");
        assert_eq!(map_column_title("status"), "Status");
        assert_eq!(map_column_title("text__1"), "Customer");
        assert_eq!(map_column_title("text2__1"), "Comment");
        assert_eq!(map_column_title("text8__1"), "Work Item");
        assert_eq!(map_column_title("unknown"), "unknown");
    }

    #[test]
    fn test_display_functions_do_not_panic() {
        // Test that display functions don't panic with empty data
        let empty_items: Vec<Item> = Vec::new();
        let empty_date_range: Vec<NaiveDate> = Vec::new();

        // These should not panic
        display_simplified_table(
            &empty_items,
            &empty_date_range,
            "test_user",
            false,
            true,
            false,
        );
        display_detailed_items(&empty_items, None, "test_user", 0, 10, true, &None, &None);
    }

    #[test]
    fn test_date_filtering_edge_cases() {
        // Test with empty items
        let empty_items: Vec<Item> = Vec::new();
        let date_range = vec![NaiveDate::from_ymd_opt(2025, 9, 15).unwrap()];

        let filtered: Vec<Item> = empty_items
            .into_iter()
            .filter(|item| is_item_matching_date_range(item, &date_range))
            .collect();

        assert_eq!(filtered.len(), 0);
    }
}
