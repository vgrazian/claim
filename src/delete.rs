use crate::monday::{MondayClient, MondayUser};
use anyhow::{anyhow, Result};
use std::io;

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

    match client.get_item_by_id(&delete_id, verbose).await {
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
    match client.delete_item(&delete_id, verbose).await {
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

        if verbose {
            if is_user_item && item_date == normalized_date {
                println!(
                    "  Checking item: date={}, customer='{}', work_item='{}'",
                    item_date, item_customer, item_work_item
                );
            }
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
