use crate::monday::{Board, Item, MondayClient, MondayUser};
use crate::{
    calculate_working_dates, map_activity_value_to_name, normalize_date, truncate_string,
    validate_date,
};
use anyhow::Result;
use chrono::prelude::*;

pub async fn handle_query_command(
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
    }

    // First, get the current year's group ID
    let current_year = get_current_year().to_string();
    let board = client
        .get_board_with_groups(board_id, verbose)
        .await?;
    
    let group_id = get_year_group_id(&board, &current_year);

    if verbose {
        println!("Using group ID: {} for year: {}", group_id, current_year);
    }

    // Search in current year group first
    let mut all_items = Vec::new();
    let mut warning_message = None;

    if verbose {
        println!("Found {} groups on board:", board.groups.as_ref().map_or(0, |g| g.len()));
        if let Some(groups) = &board.groups {
            for group in groups {
                println!("  - {} (ID: {})", group.title, group.id);
            }
        }
    }

    // Search in current year group first
    if let Some(groups) = &board.groups {
        if let Some(current_group) = groups.iter().find(|g| g.title == current_year) {
            if verbose {
                println!("\n🔍 Searching in current year group: {}", current_year);
            }

            let items = client
                .query_all_items_in_group(board_id, &current_group.id, 5000, verbose)
                .await?;

            let user_items: Vec<Item> = items
                .into_iter()
                .filter(|item| is_user_item(item, user.id, &user.name, &user.email))
                .collect();

            if !user_items.is_empty() {
                let user_items_len = user_items.len();
                all_items.extend(user_items);
                if verbose {
                    println!("✅ Found {} items in current year group", user_items_len);
                }
            } else {
                if verbose {
                    println!("❌ No items found in current year group");
                }
            }
        }
    }

    // If no items found in current group, search other groups
    if all_items.is_empty() {
        if verbose {
            println!("\n🔍 No items found in current year, searching all groups...");
        }

        if let Some(groups) = &board.groups {
            for group in groups {
                if group.title != current_year {
                    if verbose {
                        println!("  Searching in group: {}", group.title);
                    }

                    let items = client
                        .query_all_items_in_group(board_id, &group.id, 5000, verbose)
                        .await?;

                    let user_items: Vec<Item> = items
                        .into_iter()
                        .filter(|item| is_user_item(item, user.id, &user.name, &user.email))
                        .collect();

                    if !user_items.is_empty() {
                        let user_items_len = user_items.len();
                        all_items.extend(user_items);
                        if verbose {
                            println!("    Found {} items in group {}", user_items_len, group.title);
                        }
                    }
                }
            }
        }

        // Set warning if we found items in other groups but not in current group
        if !all_items.is_empty() {
            warning_message = Some(format!(
                "⚠️  WARNING: No claims found for current year ({}). Showing results from other years.",
                current_year
            ));
        }
    }

    if verbose {
        println!("\n=== Raw items found across all groups: {} ===", all_items.len());
    }

    // Then filter by date range if date filter is provided
    let filtered_items: Vec<Item> = if start_date.is_some() && !date_range.is_empty() {
        let filtered: Vec<Item> = all_items
            .into_iter()
            .filter(|item| is_item_matching_date_range(item, &date_range))
            .collect();

        if verbose {
            println!("After date range filtering: {} items", filtered.len());
        }
        filtered
    } else {
        all_items
    };

    let limited_items: Vec<Item> = filtered_items.iter().take(limit).cloned().collect();
    let filtered_items_len = filtered_items.len();

    // Display warning if applicable
    if let Some(warning) = warning_message {
        println!("\n{}", warning);
    }

    // Display the results
    if !filtered_items.is_empty() {
        if let Some(_start_date_val) = start_date {
            if target_days > 1 {
                // Multi-day query - show simplified table
                display_simplified_table(&filtered_items, &date_range, &user.name, verbose);
            } else {
                // Single day query - show detailed format
                display_detailed_items(
                    &limited_items,
                    start_date,
                    &user.name,
                    filtered_items_len,
                    limit,
                );
            }
        } else {
            // No date filter - show detailed format
            display_detailed_items(&limited_items, None, &user.name, filtered_items_len, limit);
        }
    } else {
        println!("\nNo items found for user '{}'", user.name);
        if let Some(_start_date_val) = start_date {
            if target_days > 1 {
                let end_date = date_range
                    .last()
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_default();
                println!(
                    "Date range: {} to {} ({} working days)",
                    _start_date_val.format("%Y-%m-%d"),
                    end_date,
                    target_days
                );
            } else {
                println!("Date filter: {}", _start_date_val.format("%Y-%m-%d"));
            }
        }
        
        if verbose {
            println!("\n=== DEBUG INFO ===");
            println!("Searched across all groups on the board");
            
            // Check what groups are available
            if let Some(groups) = &board.groups {
                println!("Available groups: {:?}", groups.iter().map(|g| &g.title).collect::<Vec<_>>());
            }
        }
    }

    if let Some(_start_date_val) = start_date {
        if target_days > 1 {
            let end_date = date_range
                .last()
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default();
            println!(
                "\n✅ Found {} total items matching date range: {} to {}",
                filtered_items_len,
                _start_date_val.format("%Y-%m-%d"),
                end_date
            );
        } else {
            println!(
                "\n✅ Found {} total items matching date filter: {}",
                filtered_items_len,
                _start_date_val.format("%Y-%m-%d")
            );
        }
    }

    Ok(())
}

// Rest of the file remains the same...
// Helper function to get year group ID
fn get_year_group_id(board: &Board, year: &str) -> String {
    if let Some(groups) = &board.groups {
        for group in groups {
            if group.title == year {
                return group.id.clone();
            }
        }
    }
    // Fallback to default group ID
    "new_group_mkkbbd2q".to_string()
}

// Improved helper function to check if an item belongs to a user
// Now checks for both user ID, name, and email
fn is_user_item(item: &Item, user_id: i64, user_name: &str, user_email: &str) -> bool {
    for col in &item.column_values {
        if let Some(col_id) = &col.id {
            if col_id == "person" {
                // Method 1: Check by user ID in the JSON value
                if let Some(value) = &col.value {
                    if let Ok(parsed_value) = serde_json::from_str::<serde_json::Value>(value) {
                        if let Some(persons) = parsed_value.get("personsAndTeams") {
                            if let Some(persons_array) = persons.as_array() {
                                for person in persons_array {
                                    // Check by user ID
                                    if let Some(person_id) = person.get("id").and_then(|id| id.as_i64()) {
                                        if person_id == user_id {
                                            return true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Method 2: Check by user name or email in the text field
                if let Some(text) = &col.text {
                    if !text.is_empty() && text != "null" {
                        // Check if text contains user name
                        if text.contains(user_name) {
                            return true;
                        }
                        
                        // Check if text contains user email (or part of it)
                        if text.contains(&user_email) || 
                           text.contains("valerio.graziani") || // partial email match
                           text.contains("graziani@") { // common email pattern
                            return true;
                        }
                        
                        // Also check for common variations
                        if user_name.contains(" ") {
                            let parts: Vec<&str> = user_name.split(' ').collect();
                            if parts.len() >= 2 {
                                // Check for "First Last" format
                                if text.contains(&format!("{} {}", parts[0], parts[1])) {
                                    return true;
                                }
                                // Check for "Last, First" format
                                if text.contains(&format!("{}, {}", parts[1], parts[0])) {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    false
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
                                        if let Some(name) = first_person.get("name").and_then(|n| n.as_str()) {
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

// Display simplified table for multi-day queries
fn display_simplified_table(
    items: &[Item],
    date_range: &[NaiveDate],
    user_name: &str,
    verbose: bool,
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

    if verbose {
        println!("Processing {} items across {} dates", items.len(), date_range.len());
    }

    // Create a table header with Status column
    println!(
        "\n{:<12} {:<12} {:<20} {:<15} {:<6}",
        "Date", "Status", "Customer", "Work Item", "Hours"
    );
    println!("{}", "-".repeat(70));

    // Group items by date using a HashMap
    let mut items_by_date: std::collections::HashMap<String, Vec<&Item>> =
        std::collections::HashMap::new();

    for item in items {
        if let Some(item_date) = extract_item_date(item) {
            items_by_date
                .entry(item_date)
                .or_insert_with(Vec::new)
                .push(item);
        }
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

                println!(
                    "{:<12} {:<12} {:<20} {:<15} {:<6}",
                    date_str,
                    truncate_string(&status, 10),
                    truncate_string(&customer, 18),
                    truncate_string(&work_item, 13),
                    hours_str
                );
            }
        } else {
            // Show empty row for dates with no entries
            println!(
                "{:<12} {:<12} {:<20} {:<15} {:<6}",
                date_str, "-", "-", "-", "-"
            );
        }
    }

    println!("{}", "-".repeat(70));
    println!(
        "{:<12} {:<12} {:<20} {:<15} {:<6.1}",
        "TOTAL", "", "", "", total_hours
    );
    println!(
        "\nFound {} items across {} days",
        displayed_items,
        date_range.len()
    );
}

// Helper function to display detailed items (original format)
fn display_detailed_items(
    items: &[Item],
    filter_date: Option<NaiveDate>,
    user_name: &str,
    filtered_items_len: usize,
    limit: usize,
) {
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

// Helper function to extract date from an item
pub fn extract_item_date(item: &Item) -> Option<String> {
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
                                    if let Ok(naive_date) =
                                        chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                                    {
                                        return Some(naive_date.format("%Y-%m-%d").to_string());
                                    }
                                    // Try other common date formats
                                    if let Ok(naive_date) =
                                        chrono::NaiveDate::parse_from_str(date_str, "%Y/%m/%d")
                                    {
                                        return Some(naive_date.format("%Y-%m-%d").to_string());
                                    }
                                    if let Ok(naive_date) =
                                        chrono::NaiveDate::parse_from_str(date_str, "%Y.%m.%d")
                                    {
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
                        if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(text, "%Y-%m-%d")
                        {
                            return Some(naive_date.format("%Y-%m-%d").to_string());
                        }
                        if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(text, "%Y/%m/%d")
                        {
                            return Some(naive_date.format("%Y-%m-%d").to_string());
                        }
                        if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(text, "%Y.%m.%d")
                        {
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
        assert_eq!(map_column_title("text__1"), "Text");
        assert_eq!(map_column_title("unknown"), "unknown");
    }

    #[test]
    fn test_display_functions_do_not_panic() {
        // Test that display functions don't panic with empty data
        let empty_items: Vec<Item> = Vec::new();
        let empty_date_range: Vec<NaiveDate> = Vec::new();

        // These should not panic
        display_simplified_table(&empty_items, &empty_date_range, "test_user", false);
        display_detailed_items(&empty_items, None, "test_user", 0, 10);
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