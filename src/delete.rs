use crate::monday::{MondayClient, MondayUser};
use anyhow::{anyhow, Result};
use std::io;

#[allow(clippy::too_many_arguments)]
pub async fn handle_delete_command(
    client: &MondayClient,
    user: &MondayUser,
    current_year: &str,
    delete_id: Option<String>,
    date: Option<String>,
    customer: Option<String>,
    work_item: Option<String>,
    yes: bool,
    verbose: bool,
) -> Result<()> {
    // Validate input: either delete_id OR (date + customer + work_item) must be provided
    if delete_id.is_none() && (date.is_none() || customer.is_none() || work_item.is_none()) {
        return Err(anyhow!(
            "You must provide either:\n  1. Item ID (-x/--id)\n  2. Date (-D/--date) + Customer (-c/--customer) + Work Item (-w/--wi)"
        ));
    }

    if delete_id.is_some() && (date.is_some() || customer.is_some() || work_item.is_some()) {
        return Err(anyhow!(
            "Cannot specify both item ID and date/customer/work_item filters. Choose one method."
        ));
    }

    // If delete_id is provided, use the existing logic
    if let Some(id) = delete_id {
        return delete_by_id(client, user, &id, yes, verbose).await;
    }

    // Otherwise, search for items matching date + customer + work_item
    delete_by_criteria(
        client,
        user,
        current_year,
        date.as_ref().unwrap(),
        customer.as_ref().unwrap(),
        work_item.as_ref().unwrap(),
        yes,
        verbose,
    )
    .await
}

async fn delete_by_id(
    client: &MondayClient,
    user: &MondayUser,
    delete_id: &str,
    yes: bool,
    verbose: bool,
) -> Result<()> {
    println!("\n=== Delete Claim Item ===");
    println!("User: {} ({})", user.name, user.email);
    println!("Item ID to delete: {}", delete_id);

    // First, try to get the item details to show the user what they're deleting
    if verbose {
        println!("🔍 Fetching item details...");
    }

    match client.get_item_by_id(delete_id, verbose).await {
        Ok(Some(item)) => {
            println!("\n📋 Item Details:");
            println!("  Name: {}", item.name.as_deref().unwrap_or("Unnamed"));
            println!("  ID: {}", item.id.as_deref().unwrap_or("Unknown"));

            // Show relevant column values
            if !item.column_values.is_empty() {
                println!("  Columns:");
                for col in &item.column_values {
                    if let Some(col_id) = &col.id {
                        let column_title = match col_id.as_str() {
                            "date4" => "Date",
                            "status" => "Status",
                            "text__1" => "Customer",
                            "text8__1" => "Work Item",
                            "numbers__1" => "Hours",
                            _ => continue, // Skip less important columns
                        };

                        if let Some(value) = &col.value {
                            if !value.is_empty() && value != "null" {
                                // Try to parse JSON values for better display
                                if let Ok(parsed_value) =
                                    serde_json::from_str::<serde_json::Value>(value)
                                {
                                    if let Some(date_obj) = parsed_value.get("date") {
                                        if let Some(date_str) = date_obj.as_str() {
                                            println!("    {}: {}", column_title, date_str);
                                            continue;
                                        }
                                    }
                                    if let Some(status_index) = parsed_value.get("index") {
                                        if let Some(index_num) = status_index.as_u64() {
                                            let status_name =
                                                crate::map_activity_value_to_name(index_num as u8);
                                            println!("    {}: {}", column_title, status_name);
                                            continue;
                                        }
                                    }
                                }
                                println!("    {}: {}", column_title, value);
                            }
                        } else if let Some(text) = &col.text {
                            if !text.is_empty() && text != "null" {
                                println!("    {}: {}", column_title, text);
                            }
                        }
                    }
                }
            }
        }
        Ok(None) => {
            println!("❌ Item with ID '{}' not found.", delete_id);
            return Ok(());
        }
        Err(e) => {
            println!("⚠️  Could not fetch item details: {}", e);
            println!("Proceeding with deletion based on ID only...");
        }
    }

    // Ask for confirmation unless -y flag is used
    if !yes {
        println!("\n🗑️  Are you sure you want to delete this item?");
        println!("This action cannot be undone! (y/N)");

        let mut confirmation = String::new();
        io::stdin().read_line(&mut confirmation)?;

        if confirmation.trim().to_lowercase() != "y" {
            println!("Deletion cancelled.");
            return Ok(());
        }
    }

    // Perform the deletion
    println!("\n🔄 Deleting item...");
    match client.delete_item(delete_id, verbose).await {
        Ok(_) => {
            println!("✅ Item deleted successfully!");
        }
        Err(e) => {
            println!("❌ Failed to delete item: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn delete_by_criteria(
    client: &MondayClient,
    user: &MondayUser,
    current_year: &str,
    date: &str,
    customer: &str,
    work_item: &str,
    yes: bool,
    verbose: bool,
) -> Result<()> {
    println!("\n=== Delete Claim Item by Criteria ===");
    println!("User: {} ({})", user.name, user.email);
    println!("Searching for items matching:");
    println!("  Date: {}", date);
    println!("  Customer: {}", customer);
    println!("  Work Item: {}", work_item);

    // Validate and normalize the date
    crate::validate_date(date)?;
    let normalized_date = crate::normalize_date(date);

    if verbose {
        println!("🔍 Querying items for date: {}", normalized_date);
    }

    // Query items for the specified date
    let board = client
        .query_board_verbose("6500270039", current_year, user.id, 1000, verbose)
        .await?;

    let group_id = crate::get_year_group_id(&board, current_year);

    if verbose {
        println!("Found group '{}' with ID: {}", current_year, group_id);
    }

    // Get all items for the user
    let items = client
        .query_all_items_in_group("6500270039", &group_id, 1000, verbose)
        .await?;

    if verbose {
        println!("Retrieved {} total items", items.len());
    }

    // Filter items by date, customer, and work item
    let mut matching_items = Vec::new();

    for item in items {
        // Check if item belongs to the current user
        let mut is_user_item = false;
        let mut item_date = String::new();

        // Extract values using helper function
        let item_customer = extract_column_value(&item, "text__1");
        let item_work_item = extract_column_value(&item, "text8__1");

        for col in &item.column_values {
            if let Some(col_id) = &col.id {
                match col_id.as_str() {
                    "person" => {
                        if let Some(value) = &col.value {
                            if value.contains(&user.id.to_string()) {
                                is_user_item = true;
                            }
                        }
                    }
                    "date4" => {
                        if let Some(value) = &col.value {
                            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(value) {
                                if let Some(date_str) = parsed.get("date").and_then(|d| d.as_str())
                                {
                                    item_date = date_str.to_string();
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if verbose && is_user_item && item_date == normalized_date {
            println!(
                "  Checking item: date={}, customer='{}', work_item='{}'",
                item_date, item_customer, item_work_item
            );
        }

        // Check if all criteria match
        if is_user_item
            && item_date == normalized_date
            && item_customer.eq_ignore_ascii_case(customer)
            && item_work_item.eq_ignore_ascii_case(work_item)
        {
            matching_items.push(item);
        }
    }

    if matching_items.is_empty() {
        println!("❌ No items found matching the specified criteria.");
        return Ok(());
    }

    println!("\n📋 Found {} matching item(s):", matching_items.len());
    for (i, item) in matching_items.iter().enumerate() {
        println!("\n{}. Item Details:", i + 1);
        println!("   ID: {}", item.id.as_deref().unwrap_or("Unknown"));
        println!("   Name: {}", item.name.as_deref().unwrap_or("Unnamed"));
        println!("   Date: {}", normalized_date);
        println!("   Customer: {}", customer);
        println!("   Work Item: {}", work_item);
    }

    // Ask for confirmation unless -y flag is used
    if !yes {
        println!(
            "\n🗑️  Are you sure you want to delete {} item(s)?",
            matching_items.len()
        );
        println!("This action cannot be undone! (y/N)");

        let mut confirmation = String::new();
        io::stdin().read_line(&mut confirmation)?;

        if confirmation.trim().to_lowercase() != "y" {
            println!("Deletion cancelled.");
            return Ok(());
        }
    }

    // Delete all matching items
    println!("\n🔄 Deleting {} item(s)...", matching_items.len());
    let mut deleted_count = 0;
    let mut failed_count = 0;

    for item in &matching_items {
        if let Some(item_id) = &item.id {
            match client.delete_item(item_id, verbose).await {
                Ok(_) => {
                    println!("✅ Deleted item ID: {}", item_id);
                    deleted_count += 1;
                }
                Err(e) => {
                    println!("❌ Failed to delete item ID {}: {}", item_id, e);
                    failed_count += 1;
                }
            }
        }
    }

    println!(
        "\n🎉 Deletion complete: {} deleted, {} failed",
        deleted_count, failed_count
    );

    if failed_count > 0 {
        return Err(anyhow!("Some items failed to delete"));
    }

    Ok(())
}

// Helper function to extract specific column value (same logic as in query.rs)
fn extract_column_value(item: &crate::monday::Item, column_id: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monday::{ColumnValue, Item};

    // Helper function to create a test user
    fn create_test_user() -> MondayUser {
        MondayUser {
            id: 12345,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        }
    }

    // Helper function to create a test item
    fn create_test_item(id: &str, date: &str, customer: &str, work_item: &str) -> Item {
        Item {
            id: Some(id.to_string()),
            name: Some("Test Item".to_string()),
            column_values: vec![
                ColumnValue {
                    id: Some("date4".to_string()),
                    value: Some(format!(r#"{{"date":"{}"}}"#, date)),
                    text: Some(date.to_string()),
                },
                ColumnValue {
                    id: Some("text__1".to_string()),
                    value: Some(customer.to_string()),
                    text: Some(customer.to_string()),
                },
                ColumnValue {
                    id: Some("text8__1".to_string()),
                    value: Some(work_item.to_string()),
                    text: Some(work_item.to_string()),
                },
                ColumnValue {
                    id: Some("person".to_string()),
                    value: Some(
                        r#"{"personsAndTeams":[{"id":12345,"kind":"person"}]}"#.to_string(),
                    ),
                    text: None,
                },
                ColumnValue {
                    id: Some("status".to_string()),
                    value: Some(r#"{"index":1}"#.to_string()),
                    text: Some("billable".to_string()),
                },
                ColumnValue {
                    id: Some("numbers__1".to_string()),
                    value: Some("8".to_string()),
                    text: Some("8".to_string()),
                },
            ],
        }
    }

    #[test]
    fn test_extract_column_value_text() {
        let item = create_test_item("123", "2025-01-15", "ACME Corp", "PROJ-001");

        let customer = extract_column_value(&item, "text__1");
        assert_eq!(customer, "ACME Corp");

        let work_item = extract_column_value(&item, "text8__1");
        assert_eq!(work_item, "PROJ-001");
    }

    #[test]
    fn test_extract_column_value_missing() {
        let item = create_test_item("123", "2025-01-15", "ACME Corp", "PROJ-001");

        let missing = extract_column_value(&item, "nonexistent_column");
        assert_eq!(missing, "");
    }

    #[test]
    fn test_extract_column_value_empty() {
        let mut item = create_test_item("123", "2025-01-15", "ACME Corp", "PROJ-001");
        item.column_values.push(ColumnValue {
            id: Some("empty_col".to_string()),
            value: Some("".to_string()),
            text: None,
        });

        let empty = extract_column_value(&item, "empty_col");
        assert_eq!(empty, "");
    }

    #[test]
    fn test_extract_column_value_null() {
        let mut item = create_test_item("123", "2025-01-15", "ACME Corp", "PROJ-001");
        item.column_values.push(ColumnValue {
            id: Some("null_col".to_string()),
            value: Some("null".to_string()),
            text: None,
        });

        let null_val = extract_column_value(&item, "null_col");
        assert_eq!(null_val, "");
    }

    #[test]
    fn test_extract_column_value_json_string() {
        let mut item = create_test_item("123", "2025-01-15", "ACME Corp", "PROJ-001");
        item.column_values.push(ColumnValue {
            id: Some("json_col".to_string()),
            value: Some(r#""test_value""#.to_string()),
            text: None,
        });

        let json_val = extract_column_value(&item, "json_col");
        assert_eq!(json_val, "test_value");
    }

    #[test]
    fn test_extract_column_value_uses_text_fallback() {
        let mut item = create_test_item("123", "2025-01-15", "ACME Corp", "PROJ-001");
        item.column_values.push(ColumnValue {
            id: Some("text_col".to_string()),
            value: None,
            text: Some("fallback_text".to_string()),
        });

        let text_val = extract_column_value(&item, "text_col");
        assert_eq!(text_val, "fallback_text");
    }

    #[tokio::test]
    async fn test_handle_delete_command_missing_id_and_criteria() {
        let client = MondayClient::new("test_key".to_string());
        let user = create_test_user();

        let result =
            handle_delete_command(&client, &user, "2025", None, None, None, None, false, false)
                .await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("You must provide either"));
        assert!(err_msg.contains("Item ID"));
        assert!(err_msg.contains("Date"));
    }

    #[tokio::test]
    async fn test_handle_delete_command_partial_criteria() {
        let client = MondayClient::new("test_key".to_string());
        let user = create_test_user();

        // Only date provided
        let result = handle_delete_command(
            &client,
            &user,
            "2025",
            None,
            Some("2025-01-15".to_string()),
            None,
            None,
            false,
            false,
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("You must provide either"));
    }

    #[tokio::test]
    async fn test_handle_delete_command_both_id_and_criteria() {
        let client = MondayClient::new("test_key".to_string());
        let user = create_test_user();

        let result = handle_delete_command(
            &client,
            &user,
            "2025",
            Some("123".to_string()),
            Some("2025-01-15".to_string()),
            Some("ACME".to_string()),
            Some("PROJ-001".to_string()),
            false,
            false,
        )
        .await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Cannot specify both"));
    }

    #[test]
    fn test_extract_column_value_complex_json() {
        let item = Item {
            id: Some("123".to_string()),
            name: Some("Test".to_string()),
            column_values: vec![ColumnValue {
                id: Some("complex".to_string()),
                value: Some(r#"{"nested":{"value":"deep"}}"#.to_string()),
                text: None,
            }],
        };

        // Should return the raw JSON string since it's not a simple string
        let val = extract_column_value(&item, "complex");
        assert!(val.contains("nested"));
    }

    #[test]
    fn test_extract_column_value_date_format() {
        let item = create_test_item("123", "2025-01-15", "ACME", "PROJ");

        // The date column should have the JSON format
        let date_col = item
            .column_values
            .iter()
            .find(|c| c.id.as_deref() == Some("date4"))
            .unwrap();

        assert!(date_col.value.as_ref().unwrap().contains("date"));
        assert!(date_col.value.as_ref().unwrap().contains("2025-01-15"));
    }

    #[test]
    fn test_extract_column_value_person_format() {
        let item = create_test_item("123", "2025-01-15", "ACME", "PROJ");

        // The person column should have the JSON format with user ID
        let person_col = item
            .column_values
            .iter()
            .find(|c| c.id.as_deref() == Some("person"))
            .unwrap();

        assert!(person_col.value.as_ref().unwrap().contains("12345"));
        assert!(person_col
            .value
            .as_ref()
            .unwrap()
            .contains("personsAndTeams"));
    }

    #[test]
    fn test_extract_column_value_status_format() {
        let item = create_test_item("123", "2025-01-15", "ACME", "PROJ");

        // The status column should have the JSON format with index
        let status_col = item
            .column_values
            .iter()
            .find(|c| c.id.as_deref() == Some("status"))
            .unwrap();

        assert!(status_col.value.as_ref().unwrap().contains("index"));
        assert_eq!(status_col.text.as_deref(), Some("billable"));
    }

    #[test]
    fn test_extract_column_value_numbers() {
        let item = create_test_item("123", "2025-01-15", "ACME", "PROJ");

        let hours = extract_column_value(&item, "numbers__1");
        assert_eq!(hours, "8");
    }

    #[test]
    fn test_extract_column_value_case_sensitivity() {
        let item = create_test_item("123", "2025-01-15", "ACME Corp", "PROJ-001");

        // Column IDs should be case-sensitive
        let val1 = extract_column_value(&item, "text__1");
        let val2 = extract_column_value(&item, "TEXT__1");

        assert_eq!(val1, "ACME Corp");
        assert_eq!(val2, ""); // Different case should not match
    }

    #[test]
    fn test_extract_column_value_whitespace() {
        let mut item = create_test_item("123", "2025-01-15", "ACME", "PROJ");
        item.column_values.push(ColumnValue {
            id: Some("whitespace".to_string()),
            value: Some("  trimmed  ".to_string()),
            text: None,
        });

        let val = extract_column_value(&item, "whitespace");
        // Should return the value as-is (not trimmed)
        assert_eq!(val, "  trimmed  ");
    }

    #[test]
    fn test_extract_column_value_special_characters() {
        let mut item = create_test_item("123", "2025-01-15", "ACME", "PROJ");
        item.column_values.push(ColumnValue {
            id: Some("special".to_string()),
            value: Some("Test & Co. <tag>".to_string()),
            text: None,
        });

        let val = extract_column_value(&item, "special");
        assert_eq!(val, "Test & Co. <tag>");
    }

    #[test]
    fn test_extract_column_value_unicode() {
        let mut item = create_test_item("123", "2025-01-15", "ACME", "PROJ");
        item.column_values.push(ColumnValue {
            id: Some("unicode".to_string()),
            value: Some("Café ☕ 日本語".to_string()),
            text: None,
        });

        let val = extract_column_value(&item, "unicode");
        assert_eq!(val, "Café ☕ 日本語");
    }

    #[test]
    fn test_extract_column_value_empty_column_values() {
        let item = Item {
            id: Some("123".to_string()),
            name: Some("Test".to_string()),
            column_values: vec![],
        };

        let val = extract_column_value(&item, "any_column");
        assert_eq!(val, "");
    }

    #[test]
    fn test_extract_column_value_multiple_same_id() {
        let mut item = create_test_item("123", "2025-01-15", "ACME", "PROJ");
        // Add duplicate column ID (should return first match)
        item.column_values.push(ColumnValue {
            id: Some("text__1".to_string()),
            value: Some("Second Value".to_string()),
            text: None,
        });

        let val = extract_column_value(&item, "text__1");
        assert_eq!(val, "ACME"); // Should return first match
    }
}
